package auth

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
)

// ContextKey is the Gin context key under which validated Claims are stored.
const ContextKey = "auth.claims"

// Middleware returns a Gin middleware that enforces Bearer token auth on the
// routes it is attached to. When enabled is false it is a no-op so the server
// keeps its current permissive behavior in development unless the operator
// explicitly opts in.
//
// The optional requiredScope, if non-empty, restricts access to tokens whose
// scope list contains the named scope.
func Middleware(validator *Validator, logger *logrus.Logger, enabled bool, requiredScope string) gin.HandlerFunc {
	return func(c *gin.Context) {
		// Auth disabled → pass through (dev-friendly default; matches
		// config.go:140 default of server.auth.enabled=false).
		if !enabled || validator == nil {
			c.Next()
			return
		}

		header := c.GetHeader("Authorization")
		if header == "" {
			abortUnauthorized(c, "missing Authorization header")
			return
		}

		const prefix = "Bearer "
		if !strings.HasPrefix(header, prefix) {
			abortUnauthorized(c, "Authorization header must use Bearer scheme")
			return
		}
		token := strings.TrimSpace(header[len(prefix):])

		claims, err := validator.Validate(token)
		if err != nil {
			if logger != nil {
				logger.WithFields(logrus.Fields{
					"client_ip": c.ClientIP(),
					"path":      c.Request.URL.Path,
				}).Warnf("auth rejected: %v", err)
			}
			abortUnauthorized(c, "invalid token")
			return
		}

		if requiredScope != "" && !HasScope(claims.Scope, requiredScope) {
			abortForbidden(c, "token missing required scope: "+requiredScope)
			return
		}

		// Stash claims for downstream handlers.
		c.Set(ContextKey, claims)
		c.Next()
	}
}

// ClaimsFrom returns the validated claims stored on a Gin context, if any.
func ClaimsFrom(c *gin.Context) (*Claims, bool) {
	v, ok := c.Get(ContextKey)
	if !ok {
		return nil, false
	}
	claims, ok := v.(*Claims)
	return claims, ok
}

func abortUnauthorized(c *gin.Context, msg string) {
	c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
		"error":  "unauthorized",
		"detail": msg,
	})
}

func abortForbidden(c *gin.Context, msg string) {
	c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
		"error":  "forbidden",
		"detail": msg,
	})
}
