package audit

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// captureLogger returns a logrus.Logger writing JSON records into the provided
// buffer at debug level so we can inspect emitted fields.
func captureLogger(buf *bytes.Buffer) *logrus.Logger {
	l := logrus.New()
	l.SetOutput(buf)
	l.SetLevel(logrus.DebugLevel)
	l.SetFormatter(&logrus.JSONFormatter{})
	return l
}

func newRouter(buf *bytes.Buffer) *gin.Engine {
	gin.SetMode(gin.TestMode)
	r := gin.New()
	a := New(captureLogger(buf))
	r.Use(a.Middleware())
	r.GET("/items", func(c *gin.Context) { c.JSON(200, gin.H{}) })
	r.POST("/items", func(c *gin.Context) { c.JSON(201, gin.H{}) })
	r.PUT("/items/:id", func(c *gin.Context) { c.JSON(200, gin.H{}) })
	r.PATCH("/items/:id", func(c *gin.Context) { c.JSON(200, gin.H{}) })
	r.DELETE("/items/:id", func(c *gin.Context) { c.JSON(204, gin.H{}) })
	r.POST("/fail", func(c *gin.Context) { c.JSON(500, gin.H{}) })
	return r
}

func findAuditRecord(t *testing.T, buf *bytes.Buffer, path string) map[string]interface{} {
	t.Helper()
	for _, line := range strings.Split(strings.TrimSpace(buf.String()), "\n") {
		if line == "" {
			continue
		}
		var rec map[string]interface{}
		require.NoError(t, json.Unmarshal([]byte(line), &rec), "log line is not JSON: %s", line)
		if rec["audit"] == true && rec["path"] == path {
			return rec
		}
	}
	return nil
}

func TestMiddleware_TagsRequestID(t *testing.T) {
	var buf bytes.Buffer
	r := newRouter(&buf)

	// Inbound X-Request-Id is honoured.
	req := httptest.NewRequest("GET", "/items", nil)
	req.Header.Set("X-Request-Id", "caller-supplied-123")
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)

	assert.Equal(t, http.StatusOK, rr.Code)
	assert.Equal(t, "caller-supplied-123", rr.Header().Get("X-Request-Id"))
}

func TestMiddleware_GeneratesRequestIDWhenAbsent(t *testing.T) {
	var buf bytes.Buffer
	r := newRouter(&buf)
	req := httptest.NewRequest("GET", "/items", nil)
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	got := rr.Header().Get("X-Request-Id")
	assert.NotEmpty(t, got, "X-Request-Id must be generated when not supplied")
	assert.Greater(t, len(got), 8)
}

func TestMiddleware_AuditsStateChangingMethods(t *testing.T) {
	cases := []struct {
		method     string
		path       string
		status     int
		wantAction string
	}{
		{http.MethodPost, "/items", http.StatusCreated, "create"},
		{http.MethodPut, "/items/42", http.StatusOK, "update"},
		{http.MethodPatch, "/items/42", http.StatusOK, "update"},
		{http.MethodDelete, "/items/42", http.StatusNoContent, "delete"},
	}

	for _, tc := range cases {
		t.Run(tc.method, func(t *testing.T) {
			var buf bytes.Buffer
			r := newRouter(&buf)

			req := httptest.NewRequest(tc.method, tc.path, nil)
			rr := httptest.NewRecorder()
			r.ServeHTTP(rr, req)
			assert.Equal(t, tc.status, rr.Code)

			rec := findAuditRecord(t, &buf, tc.path)
			require.NotNil(t, rec, "expected audit record for %s %s", tc.method, tc.path)
			assert.Equal(t, tc.wantAction, rec["action"])
			assert.Equal(t, tc.method, rec["method"])
			assert.EqualValues(t, tc.status, rec["status"])
			assert.NotEmpty(t, rec["request_id"])
		})
	}
}

func TestMiddleware_DoesNotAuditSafeMethods(t *testing.T) {
	var buf bytes.Buffer
	r := newRouter(&buf)

	req := httptest.NewRequest("GET", "/items", nil)
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	assert.Equal(t, http.StatusOK, rr.Code)

	rec := findAuditRecord(t, &buf, "/items")
	assert.Nil(t, rec, "GET requests must not emit audit records")
}

func TestMiddleware_EmitsErrorLevelForServerFailures(t *testing.T) {
	var buf bytes.Buffer
	r := newRouter(&buf)

	req := httptest.NewRequest("POST", "/fail", nil)
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	assert.Equal(t, http.StatusInternalServerError, rr.Code)

	rec := findAuditRecord(t, &buf, "/fail")
	require.NotNil(t, rec)
	assert.Equal(t, "error", rec["level"])
}

func TestMiddleware_LogsClientMetadata(t *testing.T) {
	var buf bytes.Buffer
	r := newRouter(&buf)

	req := httptest.NewRequest("POST", "/items", nil)
	req.Header.Set("User-Agent", "kodevibe-test/1.0")
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)

	rec := findAuditRecord(t, &buf, "/items")
	require.NotNil(t, rec)
	assert.Equal(t, "kodevibe-test/1.0", rec["user_agent"])
	assert.Contains(t, rec, "client_ip")
	assert.Contains(t, rec, "duration_ms")
}

func TestRequestIDFrom(t *testing.T) {
	gin.SetMode(gin.TestMode)
	r := gin.New()
	r.Use(New(nil).Middleware())
	r.GET("/", func(c *gin.Context) {
		id, ok := RequestIDFrom(c)
		assert.True(t, ok)
		c.Header("X-Echo-Id", id)
		c.JSON(200, gin.H{})
	})
	req := httptest.NewRequest("GET", "/", nil)
	rr := httptest.NewRecorder()
	r.ServeHTTP(rr, req)
	assert.Equal(t, rr.Header().Get("X-Request-Id"), rr.Header().Get("X-Echo-Id"))
}
