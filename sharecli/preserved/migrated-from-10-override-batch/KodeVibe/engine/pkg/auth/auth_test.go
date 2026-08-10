package auth

import (
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const testSecret = "test-secret-do-not-use-in-prod-1234567890"

func newTestValidator(t *testing.T) *Validator {
	t.Helper()
	v, err := NewValidator(testSecret)
	require.NoError(t, err)
	return v
}

func TestNewValidator_RejectsEmptySecret(t *testing.T) {
	for _, empty := range []string{"", " ", "\t", "\n"} {
		_, err := NewValidator(empty)
		assert.Error(t, err, "expected error for empty secret %q", empty)
	}
}

func TestValidator_SignValidate_RoundTrip(t *testing.T) {
	v := newTestValidator(t)
	now := time.Now()
	tok, err := v.Sign(Claims{
		Subject: "alice",
		IssueAt: now.Unix(),
		Expires: now.Add(time.Hour).Unix(),
		Scope:   "scan:write",
	})
	require.NoError(t, err)
	assert.NotEmpty(t, tok)
	assert.True(t, strings.Contains(tok, "."))

	claims, err := v.Validate(tok)
	require.NoError(t, err)
	assert.Equal(t, "alice", claims.Subject)
	assert.Equal(t, "scan:write", claims.Scope)
	assert.Equal(t, now.Unix(), claims.IssueAt)
	assert.Equal(t, now.Add(time.Hour).Unix(), claims.Expires)
}

func TestValidator_Sign_DefaultsIAT(t *testing.T) {
	v := newTestValidator(t)
	tok, err := v.Sign(Claims{
		Subject: "alice",
		Expires: time.Now().Add(time.Minute).Unix(),
	})
	require.NoError(t, err)
	claims, err := v.Validate(tok)
	require.NoError(t, err)
	assert.WithinDuration(t, time.Now(), time.Unix(claims.IssueAt, 0), 5*time.Second)
}

func TestValidator_Sign_RejectsBadInputs(t *testing.T) {
	v := newTestValidator(t)
	now := time.Now()

	_, err := v.Sign(Claims{Subject: ""})
	assert.Error(t, err)

	_, err = v.Sign(Claims{Subject: "x", IssueAt: now.Unix(), Expires: now.Unix()})
	assert.Error(t, err, "exp must be strictly after iat")
}

func TestValidator_Sign_NilValidator(t *testing.T) {
	var v *Validator
	_, err := v.Sign(Claims{Subject: "x"})
	assert.Error(t, err)
}

func TestValidator_Validate_RejectsBadInputs(t *testing.T) {
	v := newTestValidator(t)

	cases := []struct {
		name  string
		token string
	}{
		{"empty", ""},
		{"whitespace", "   "},
		{"no dot", "abcdef"},
		{"missing sig", "abc."},
		{"missing payload", ".xyz"},
		{"bad b64 payload", "!!.xyz"},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			_, err := v.Validate(tc.token)
			assert.Error(t, err)
		})
	}
}

func TestValidator_Validate_RejectsTamperedSignature(t *testing.T) {
	v := newTestValidator(t)
	tok, err := v.Sign(Claims{
		Subject: "alice",
		Expires: time.Now().Add(time.Hour).Unix(),
	})
	require.NoError(t, err)

	parts := strings.Split(tok, ".")
	require.Len(t, parts, 2)
	last := parts[1]
	require.NotEmpty(t, last)

	flipped := last[:len(last)-1] + string(flipChar(last[len(last)-1]))
	require.NotEqual(t, last, flipped, "flipChar must produce a different byte")
	tampered := parts[0] + "." + flipped
	_, err = v.Validate(tampered)
	assert.Error(t, err, "tampered signature must be rejected")
}

func TestValidator_Validate_RejectsWrongSecret(t *testing.T) {
	signer, err := NewValidator("secret-A-12345678901234567890")
	require.NoError(t, err)
	tok, err := signer.Sign(Claims{
		Subject: "alice",
		Expires: time.Now().Add(time.Hour).Unix(),
	})
	require.NoError(t, err)

	verifier, err := NewValidator("secret-B-09876543210987654321")
	require.NoError(t, err)
	_, err = verifier.Validate(tok)
	assert.Error(t, err, "token signed with different secret must be rejected")
}

func TestValidator_Validate_RejectsExpired(t *testing.T) {
	v := newTestValidator(t)
	tok, err := v.Sign(Claims{
		Subject: "alice",
		IssueAt: time.Now().Add(-2 * time.Hour).Unix(),
		Expires: time.Now().Add(-time.Hour).Unix(),
	})
	require.NoError(t, err)
	_, err = v.Validate(tok)
	assert.Error(t, err, "expired token must be rejected")
}

func TestValidator_Validate_NilValidator(t *testing.T) {
	var v *Validator
	_, err := v.Validate("anything.atall")
	assert.Error(t, err)
}

func TestClaims_IsExpired(t *testing.T) {
	now := time.Now()
	assert.True(t, (&Claims{Expires: now.Add(-time.Minute).Unix()}).IsExpired())
	assert.False(t, (&Claims{Expires: now.Add(time.Minute).Unix()}).IsExpired())
}

func TestHasScope(t *testing.T) {
	assert.True(t, HasScope("", "anything"), "empty scope matches everything")
	assert.True(t, HasScope("read", "read"))
	assert.True(t, HasScope("read,write", "write"))
	assert.True(t, HasScope("read write", "write"))
	assert.True(t, HasScope("a, b, c", "b"))
	assert.False(t, HasScope("read", "write"))
	assert.False(t, HasScope("read,write", "delete"))
	assert.False(t, HasScope("read", ""))
	assert.False(t, HasScope("read", "READ"), "scope comparison is case-sensitive")
}

// flipChar returns a different base64url character so signature-tampering tests
// produce a well-formed but wrong token.
func flipChar(c byte) byte {
	if c == 'A' {
		return 'B'
	}
	return 'A'
}
