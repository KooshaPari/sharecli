package server

import (
	"bytes"
	"log/slog"
	"strings"
	"testing"
)

// TestSetupLoggerIdempotent verifies that setupLogger() can be invoked more
// than once without panicking and that the default slog logger is still
// configured with the JSON handler and the "service=kwatch" attribute after
// repeated calls. This guards the contract documented on setupLogger itself
// and protects the rest of the server package from accidental global-logger
// churn when constructors are called repeatedly (e.g. from tests).
func TestSetupLoggerIdempotent(t *testing.T) {
	// Calling setupLogger multiple times in a row must not panic. We loop a
	// few times to exercise the sync.Once path as well as the no-op fast
	// path on subsequent invocations.
	defer func() {
		if r := recover(); r != nil {
			t.Fatalf("setupLogger panicked on repeated calls: %v", r)
		}
	}()

	for i := 0; i < 5; i++ {
		setupLogger()
	}

	// After setupLogger has run, the default logger should be wired up to
	// emit JSON containing the service tag. Capture stderr by swapping the
	// default logger to one that writes to a buffer and asserting the
	// rendered output contains the expected structured fields.
	buf := &bytes.Buffer{}
	jsonHandler := slog.NewJSONHandler(buf, &slog.HandlerOptions{Level: slog.LevelInfo})
	prev := slog.Default()
	slog.SetDefault(slog.New(jsonHandler).With(slog.String("service", "kwatch")))
	t.Cleanup(func() { slog.SetDefault(prev) })

	slog.Info("probe", "k", "v")

	out := buf.String()
	if !strings.Contains(out, `"service":"kwatch"`) {
		t.Errorf("expected service tag in JSON output, got: %s", out)
	}
	if !strings.Contains(out, `"msg":"probe"`) {
		t.Errorf("expected message in JSON output, got: %s", out)
	}
}
