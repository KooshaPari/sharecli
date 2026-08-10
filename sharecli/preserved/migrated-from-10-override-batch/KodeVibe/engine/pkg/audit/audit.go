// Package audit provides an HTTP middleware that emits structured audit-log
// entries for state-changing API requests (POST, PUT, PATCH, DELETE). It is
// deliberately separate from the standard request logger so that audit events
// can be routed to a dedicated sink (file, SIEM, etc.) without mixing with
// general request noise.
//
// Each audit record is one JSON object containing:
//
//	timestamp   RFC3339Nano
//	method      HTTP method
//	path        request URL path
//	status      response status code
//	client_ip   X-Forwarded-For aware client IP
//	user_agent  User-Agent header
//	duration_ms wall-clock latency in milliseconds
//	subject     authenticated subject (if any, from auth context)
//	action      derived verb: "create" | "update" | "delete" | "exec"
//	request_id  per-request UUID (also echoed via X-Request-Id response header)
//
// The middleware is fail-open: a failure to emit an audit record must never
// break the underlying request. Errors are logged via the standard logger at
// warn level.
package audit

import (
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/sirupsen/logrus"

	"kodevibe/pkg/auth"
)

const (
	// RequestIDHeader is both the inbound and outbound header used to carry
	// the per-request correlation identifier.
	RequestIDHeader = "X-Request-Id"

	// ContextKey is where the audit middleware stores the request ID for
	// downstream handlers and response headers.
	ContextKey = "audit.request_id"

	// logField is the structured field name used by audit records so they can
	// be filtered downstream (e.g. by Loki, Splunk, journald).
	logField = "audit"
)

// Logger emits one audit record per state-changing request. Construct it once
// per server (or per sink) and reuse; the middleware holds only a pointer.
type Logger struct {
	logger *logrus.Logger
}

// New returns an audit logger that writes JSON records via the provided
// logrus.Logger. Pass nil to use the global standard logger.
func New(logger *logrus.Logger) *Logger {
	if logger == nil {
		logger = logrus.StandardLogger()
	}
	return &Logger{logger: logger}
}

// Middleware returns a Gin middleware that tags every request with a request ID
// and emits an audit record for state-changing methods. Safe-method requests
// (GET, HEAD, OPTIONS) are still tagged but not audit-logged here so the
// request log (L5) is not duplicated.
func (a *Logger) Middleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Honour caller-supplied correlation IDs so multi-hop chains stay
		// linkable. Generate one if absent.
		reqID := c.GetHeader(RequestIDHeader)
		if reqID == "" {
			reqID = uuid.New().String()
		}
		c.Set(ContextKey, reqID)
		c.Writer.Header().Set(RequestIDHeader, reqID)

		start := time.Now()
		c.Next()
		latency := time.Since(start)

		if !isStateChanging(c.Request.Method) {
			return
		}

		subject := "-"
		if claims, ok := auth.ClaimsFrom(c); ok && claims != nil {
			subject = claims.Subject
		}

		entry := a.logger.WithFields(logrus.Fields{
			logField:      true,
			"method":      c.Request.Method,
			"path":        c.Request.URL.Path,
			"status":      c.Writer.Status(),
			"client_ip":   c.ClientIP(),
			"user_agent":  c.Request.UserAgent(),
			"duration_ms": latency.Milliseconds(),
			"subject":     subject,
			"action":      actionFor(c.Request.Method),
			"request_id":  reqID,
		})

		switch {
		case c.Writer.Status() >= 500:
			entry.Error("audit")
		case c.Writer.Status() >= 400:
			entry.Warn("audit")
		default:
			entry.Info("audit")
		}
	}
}

// RequestIDFrom returns the per-request ID placed on a Gin context, if any.
func RequestIDFrom(c *gin.Context) (string, bool) {
	v, ok := c.Get(ContextKey)
	if !ok {
		return "", false
	}
	s, ok := v.(string)
	return s, ok
}

func isStateChanging(method string) bool {
	switch method {
	case http.MethodPost,
		http.MethodPut,
		http.MethodPatch,
		http.MethodDelete:
		return true
	default:
		return false
	}
}

func actionFor(method string) string {
	switch method {
	case http.MethodPost:
		return "create"
	case http.MethodPut, http.MethodPatch:
		return "update"
	case http.MethodDelete:
		return "delete"
	default:
		return "other"
	}
}
