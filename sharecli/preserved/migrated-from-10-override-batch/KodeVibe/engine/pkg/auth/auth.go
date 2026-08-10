// Package auth implements HMAC-SHA256 bearer-token authentication for the
// KodeVibe HTTP server.
//
// It is intentionally self-contained (no JWT library) so the runtime has no
// extra dependency surface and the token format is auditable line by line.
//
// Token format: base64url(payload) + "." + base64url(hmacSHA256(secret, payload))
// payload:      base64url( {"sub":"<subject>","exp":<unix-secs>,"iat":<unix-secs>,"scope":"<scopes>"} )
package auth

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"
)

// Claims represents the JSON payload of a bearer token.
type Claims struct {
	Subject string `json:"sub"`
	IssueAt int64  `json:"iat"`
	Expires int64  `json:"exp"`
	Scope   string `json:"scope,omitempty"`
}

// IsExpired returns true if the token's expiry is in the past (clock skew ignored
// for safety; callers may add leeway when validating against external clocks).
func (c *Claims) IsExpired() bool {
	return time.Now().Unix() >= c.Expires
}

// Validator validates HS256 bearer tokens against a shared secret.
type Validator struct {
	secret []byte
	now    func() time.Time
}

// NewValidator returns a Validator that signs and validates tokens using the
// provided shared secret. The secret must be non-empty.
func NewValidator(secret string) (*Validator, error) {
	if strings.TrimSpace(secret) == "" {
		return nil, errors.New("auth: shared secret must not be empty")
	}
	return &Validator{
		secret: []byte(secret),
		now:    time.Now,
	}, nil
}

// Sign produces a signed bearer token for the given claims. The caller is
// responsible for ensuring Claims.Expires is set to a sensible future time.
func (v *Validator) Sign(claims Claims) (string, error) {
	if v == nil {
		return "", errors.New("auth: nil validator")
	}
	if claims.Subject == "" {
		return "", errors.New("auth: subject (sub) is required")
	}
	if claims.IssueAt == 0 {
		claims.IssueAt = v.now().Unix()
	}
	if claims.Expires <= claims.IssueAt {
		return "", errors.New("auth: exp must be after iat")
	}

	payloadJSON, err := json.Marshal(claims)
	if err != nil {
		return "", fmt.Errorf("auth: marshal claims: %w", err)
	}
	payload := base64.RawURLEncoding.EncodeToString(payloadJSON)

	mac := hmac.New(sha256.New, v.secret)
	if _, err := mac.Write([]byte(payload)); err != nil {
		// hmac.Hash.Write never returns a non-nil error, but guard anyway.
		return "", fmt.Errorf("auth: hmac write: %w", err)
	}
	sig := base64.RawURLEncoding.EncodeToString(mac.Sum(nil))

	return payload + "." + sig, nil
}

// Validate parses and verifies a bearer token, returning the embedded claims on
// success. It rejects empty tokens, malformed tokens, bad signatures, and
// expired tokens.
func (v *Validator) Validate(token string) (*Claims, error) {
	if v == nil {
		return nil, errors.New("auth: nil validator")
	}
	token = strings.TrimSpace(token)
	if token == "" {
		return nil, errors.New("auth: empty token")
	}

	parts := strings.Split(token, ".")
	if len(parts) != 2 || parts[0] == "" || parts[1] == "" {
		return nil, errors.New("auth: malformed token")
	}

	mac := hmac.New(sha256.New, v.secret)
	if _, err := mac.Write([]byte(parts[0])); err != nil {
		return nil, fmt.Errorf("auth: hmac write: %w", err)
	}
	expectedSig := base64.RawURLEncoding.EncodeToString(mac.Sum(nil))

	// Constant-time comparison to avoid timing side channels.
	if !hmac.Equal([]byte(expectedSig), []byte(parts[1])) {
		return nil, errors.New("auth: invalid signature")
	}

	payloadJSON, err := base64.RawURLEncoding.DecodeString(parts[0])
	if err != nil {
		return nil, fmt.Errorf("auth: decode payload: %w", err)
	}

	var claims Claims
	if err := json.Unmarshal(payloadJSON, &claims); err != nil {
		return nil, fmt.Errorf("auth: unmarshal claims: %w", err)
	}

	if claims.Subject == "" {
		return nil, errors.New("auth: missing sub claim")
	}
	if claims.Expires == 0 {
		return nil, errors.New("auth: missing exp claim")
	}
	if claims.IsExpired() {
		return nil, errors.New("auth: token expired")
	}
	return &claims, nil
}

// HasScope reports whether the comma- or space-separated scope string contains
// the given scope token.
//
// Semantics:
//   - empty scopeList ("") -> permissive: any non-empty want is granted
//   - empty want ("")      -> denied: caller requested no permission
//   - case-sensitive match against the want token
//
// This deliberately splits the two empty cases so a caller cannot accidentally
// pass "" for `want` and bypass scope checks.
func HasScope(scopeList, want string) bool {
	if want == "" {
		return false
	}
	if scopeList == "" {
		return true
	}
	for _, s := range strings.FieldsFunc(scopeList, func(r rune) bool {
		return r == ',' || r == ' '
	}) {
		if s == want {
			return true
		}
	}
	return false
}
