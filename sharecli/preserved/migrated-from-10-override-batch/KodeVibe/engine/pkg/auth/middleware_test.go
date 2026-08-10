package auth

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newAuthTestLogger() *logrus.Logger {
	l := logrus.New()
	l.SetLevel(logrus.PanicLevel)
	return l
}

func newRouterWithAuth(t *testing.T, enabled bool, secret string, requiredScope string) (*gin.Engine, *Validator) {
	t.Helper()
	gin.SetMode(gin.TestMode)

	var validator *Validator
	if enabled {
		v, err := NewValidator(secret)
		require.NoError(t, err)
		validator = v
	}

	r := gin.New()
	r.Use(Middleware(validator, newAuthTestLogger(), enabled, requiredScope))
	r.GET("/probe", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})
	r.POST("/mutate", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"ok": true})
	})
	return r, validator
}

func TestMiddleware_Disabled_AllowsEverything(t *testing.T) {
	r, _ := newRouterWithAuth(t, false, "", "")
	for _, method := range []string{"GET", "POST"} {
		// Build the request for the route matching the method; the helper
		// router registers a GET handler on /probe and a POST handler on
		// /mutate. Use the matching URL so we exercise both verbs through
		// the middleware without hitting a 404 from Gin.
		url := "/probe"
		if method == "POST" {
			url = "/mutate"
		}
		req := httptest.NewRequest(method, url, nil)
		rr := httptest.NewRecorder()
		r.ServeHTTP(rr, req)
		assert.Equal(t, http.StatusOK, rr.Code, "method=%s url=%s", method, url)
	}
}

func TestMiddleware_Enabled_RequiresBearer(t *testing.T) {
	r, _ := newRouterWithAuth(t, true, testSecret, "")

	// Missing header.
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, httptest.NewRequest("GET", "/probe", nil))
	assert.Equal(t, http.StatusUnauthorized, rr.Code)

	// Wrong scheme.
	req := httptest.NewRequest("GET", "/probe", nil)
	req.Header.Set("Authorization", "Basic dXNlcjpwYXNz")
	rr = httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	assert.Equal(t, http.StatusUnauthorized, rr.Code)
}

func TestMiddleware_Enabled_AcceptsValidToken(t *testing.T) {
	r, v := newRouterWithAuth(t, true, testSecret, "")

	tok, err := v.Sign(Claims{
		Subject: "alice",
		Expires: time.Now().Add(time.Hour).Unix(),
	})
	require.NoError(t, err)

	req := httptest.NewRequest("GET", "/probe", nil)
	req.Header.Set("Authorization", "Bearer "+tok)
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	assert.Equal(t, http.StatusOK, rr.Code)

	// Claims should be on the context for handlers to inspect.
	var body map[string]interface{}
	require.NoError(t, json.Unmarshal(rr.Body.Bytes(), &body))
	assert.Equal(t, true, body["ok"])
}

func TestMiddleware_Enabled_RejectsExpiredToken(t *testing.T) {
	r, v := newRouterWithAuth(t, true, testSecret, "")
	// Sign a token whose lifetime is fully in the past so Sign() will accept it
	// (exp must be after iat). The validator must then reject it as expired.
	tok, err := v.Sign(Claims{
		Subject: "alice",
		IssueAt: time.Now().Add(-2 * time.Hour).Unix(),
		Expires: time.Now().Add(-time.Minute).Unix(),
	})
	require.NoError(t, err)

	req := httptest.NewRequest("GET", "/probe", nil)
	req.Header.Set("Authorization", "Bearer "+tok)
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	assert.Equal(t, http.StatusUnauthorized, rr.Code)
}

func TestMiddleware_RequiredScope_BlocksInsufficient(t *testing.T) {
	r, v := newRouterWithAuth(t, true, testSecret, "admin")
	tok, err := v.Sign(Claims{
		Subject: "alice",
		Scope:   "read,write",
		Expires: time.Now().Add(time.Hour).Unix(),
	})
	require.NoError(t, err)

	req := httptest.NewRequest("GET", "/probe", nil)
	req.Header.Set("Authorization", "Bearer "+tok)
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	assert.Equal(t, http.StatusForbidden, rr.Code)
}

func TestMiddleware_RequiredScope_AllowsMatching(t *testing.T) {
	r, v := newRouterWithAuth(t, true, testSecret, "admin")
	tok, err := v.Sign(Claims{
		Subject: "alice",
		Scope:   "read,admin,write",
		Expires: time.Now().Add(time.Hour).Unix(),
	})
	require.NoError(t, err)

	req := httptest.NewRequest("GET", "/probe", nil)
	req.Header.Set("Authorization", "Bearer "+tok)
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	assert.Equal(t, http.StatusOK, rr.Code)
}

func TestMiddleware_ClaimsFromContext(t *testing.T) {
	gin.SetMode(gin.TestMode)
	v, err := NewValidator(testSecret)
	require.NoError(t, err)

	r := gin.New()
	r.Use(Middleware(v, newAuthTestLogger(), true, ""))

	var captured *Claims
	r.GET("/probe", func(c *gin.Context) {
		captured, _ = ClaimsFrom(c)
		c.JSON(http.StatusOK, gin.H{})
	})

	tok, err := v.Sign(Claims{
		Subject: "alice",
		Scope:   "read",
		Expires: time.Now().Add(time.Hour).Unix(),
	})
	require.NoError(t, err)

	req := httptest.NewRequest("GET", "/probe", nil)
	req.Header.Set("Authorization", "Bearer "+tok)
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	require.NotNil(t, captured)
	assert.Equal(t, "alice", captured.Subject)
	assert.Equal(t, "read", captured.Scope)
}
