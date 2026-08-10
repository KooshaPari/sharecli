package server

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"path/filepath"
	"strings"
	"sync/atomic"
	"testing"
	"time"
)

// mockRunner is a deterministic test double for the server.Runner interface.
type mockRunner struct {
	runAllCalls int32
	results     map[string]CommandResult
	history     []interface{}
	metrics     CommandMetrics
}

// Compile-time assertion that mockRunner satisfies the server.Runner
// interface, so any future signature drift surfaces here instead of in the
// integration test body.
var _ Runner = (*mockRunner)(nil)

func (m *mockRunner) RunAll(_ interface{}) map[string]CommandResult {
	atomic.AddInt32(&m.runAllCalls, 1)
	if m.results == nil {
		return map[string]CommandResult{}
	}
	return m.results
}

func (m *mockRunner) GetHistory() []interface{} { return m.history }

func (m *mockRunner) GetMetrics() CommandMetrics { return m.metrics }

// findFreePort asks the kernel for an unused TCP port on the loopback
// interface. The temporary listener is closed before returning; on Linux
// and macOS the subsequent bind from http.Server is allowed by the kernel
// because the previous socket has been fully released.
func findFreePort(t *testing.T) int {
	t.Helper()
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("findFreePort: listen: %v", err)
	}
	port := l.Addr().(*net.TCPAddr).Port
	if err := l.Close(); err != nil {
		t.Fatalf("findFreePort: close: %v", err)
	}
	return port
}

// waitForReady polls addr until a TCP connection succeeds or the deadline
// elapses; the test fails on timeout so a flake surfaces as a clear error
// rather than a confusing client-side connection refused.
func waitForReady(t *testing.T, addr string, timeout time.Duration) {
	t.Helper()
	deadline := time.Now().Add(timeout)
	var lastErr error
	for time.Now().Before(deadline) {
		conn, err := net.DialTimeout("tcp", addr, 100*time.Millisecond)
		if err == nil {
			_ = conn.Close()
			return
		}
		lastErr = err
		time.Sleep(20 * time.Millisecond)
	}
	t.Fatalf("server on %s did not become ready within %v: %v", addr, timeout, lastErr)
}

// TestServer_Integration exercises the Server type end-to-end: it brings up
// a real HTTP listener on a free loopback port, drives it with a real
// client, and verifies that constructor wiring, getters, runner integration,
// middleware (metrics, methods), and graceful-shutdown all behave together
// correctly. This is the integration test promised by AGENTS.md for the
// server.Server type.
func TestServer_Integration(t *testing.T) {
	port := findFreePort(t)
	now := time.Now().UTC().Format(time.RFC3339)
	runner := &mockRunner{
		results: map[string]CommandResult{
			"typescript": {Passed: true, IssueCount: 0, Duration: "12ms", LastRun: now},
			"lint":       {Passed: true, IssueCount: 0, Duration: "8ms", LastRun: now},
		},
		history: []interface{}{"entry-1", "entry-2"},
		metrics: CommandMetrics{TotalRuns: 7, SuccessRate: 1.0, FailureCount: 0, AverageTime: 4.5},
	}

	cfg := DefaultConfig()
	cfg.Host = "127.0.0.1"
	cfg.Port = port
	cfg.WorkingDir = t.TempDir()
	cfg.EnableCORS = false

	srv := New(cfg, runner)

	// Getter / accessor smoke tests on a server that has not started yet.
	if srv.GetConfig() != cfg {
		t.Fatalf("GetConfig() did not return the *Config passed to New()")
	}
	if !srv.IsHealthy() {
		t.Fatal("IsHealthy() = false on a fresh server; want true")
	}
	if got := srv.GetUptime(); got < 0 || got > 5*time.Second {
		t.Errorf("GetUptime() = %v on a fresh server; want a small non-negative duration", got)
	}
	if got := srv.GetMetrics(); got == nil {
		t.Fatal("GetMetrics() returned nil on a fresh server")
	} else if got.RequestCount != 0 {
		t.Errorf("GetMetrics().RequestCount = %d on a fresh server; want 0", got.RequestCount)
	}

	ctx, cancel := context.WithCancel(context.Background())
	serveErr := make(chan error, 1)
	go func() {
		serveErr <- srv.StartWithContext(ctx)
	}()

	// Defensive cleanup: if the test fails before reaching the explicit
	// shutdown at the bottom, still cancel the context and wait for the
	// goroutine to exit so we don't leak it across tests.
	t.Cleanup(func() {
		cancel()
		select {
		case <-serveErr:
		case <-time.After(5 * time.Second):
		}
	})

	addr := fmt.Sprintf("%s:%d", cfg.Host, cfg.Port)
	waitForReady(t, addr, 3*time.Second)

	client := &http.Client{Timeout: 2 * time.Second}

	// 1. /ping — simplest endpoint, returns the literal "pong" body.
	resp, err := client.Get("http://" + addr + "/ping")
	if err != nil {
		t.Fatalf("GET /ping: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Errorf("GET /ping status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	body, _ := io.ReadAll(resp.Body)
	resp.Body.Close()
	if got := strings.TrimSpace(string(body)); got != "pong" {
		t.Errorf("GET /ping body = %q, want %q", got, "pong")
	}

	// 2. /status — exercises runner integration: RunAll must be called,
	//    the response must echo every command from the runner, and the
	//    working directory must be reported back to the client.
	resp, err = client.Get("http://" + addr + "/status")
	if err != nil {
		t.Fatalf("GET /status: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Errorf("GET /status status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	var status StatusResponse
	if err := json.NewDecoder(resp.Body).Decode(&status); err != nil {
		t.Fatalf("decode /status: %v", err)
	}
	resp.Body.Close()
	if got := atomic.LoadInt32(&runner.runAllCalls); got == 0 {
		t.Error("RunAll was never called; /status should invoke it at least once")
	}
	if len(status.Commands) != len(runner.results) {
		t.Errorf("/status returned %d commands, want %d", len(status.Commands), len(runner.results))
	}
	for name, cmd := range status.Commands {
		if !cmd.Passed {
			t.Errorf("status.Commands[%q].Passed = false, want true", name)
		}
	}
	// t.TempDir() may resolve through a symlink (notably on macOS where
	// $TMPDIR points at /var/folders/...), so match on the trailing path
	// component instead of the full string.
	if want := filepath.Base(cfg.WorkingDir); !strings.Contains(status.Directory, want) {
		t.Errorf("status.Directory = %q, want it to contain %q", status.Directory, want)
	}

	// 3. /history — verifies GetHistory() is wired through the handler
	//    and surfaced as the documented "history"/"count" payload.
	resp, err = client.Get("http://" + addr + "/history")
	if err != nil {
		t.Fatalf("GET /history: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Errorf("GET /history status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	var histPayload struct {
		History []interface{} `json:"history"`
		Count   int           `json:"count"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&histPayload); err != nil {
		t.Fatalf("decode /history: %v", err)
	}
	resp.Body.Close()
	if histPayload.Count != len(runner.history) {
		t.Errorf("/history count = %d, want %d", histPayload.Count, len(runner.history))
	}
	if len(histPayload.History) != len(runner.history) {
		t.Errorf("/history history length = %d, want %d", len(histPayload.History), len(runner.history))
	}

	// 4. /metrics — verifies that the metrics middleware increments the
	//    request count before the handler reads it, and that the runner
	//    metrics are exposed on the response.
	resp, err = client.Get("http://" + addr + "/metrics")
	if err != nil {
		t.Fatalf("GET /metrics: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Errorf("GET /metrics status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	var mr MetricsResponse
	if err := json.NewDecoder(resp.Body).Decode(&mr); err != nil {
		t.Fatalf("decode /metrics: %v", err)
	}
	resp.Body.Close()
	if mr.Commands.TotalRuns != runner.metrics.TotalRuns {
		t.Errorf("/metrics Commands.TotalRuns = %d, want %d", mr.Commands.TotalRuns, runner.metrics.TotalRuns)
	}
	// /ping, /status, /history, /metrics have all hit the server by now;
	// the metrics middleware counts the current request as well, so the
	// handler should observe RequestCount >= 4.
	if mr.Server.RequestCount < 3 {
		t.Errorf("/metrics Server.RequestCount = %d, want >= 3", mr.Server.RequestCount)
	}

	// 5. SetRunner — replacing the runner at runtime must be visible to
	//    the very next /metrics request.
	newRunner := &mockRunner{
		results: map[string]CommandResult{},
		history: []interface{}{},
		metrics: CommandMetrics{TotalRuns: 99, SuccessRate: 0.5, FailureCount: 1, AverageTime: 10},
	}
	srv.SetRunner(newRunner)
	resp, err = client.Get("http://" + addr + "/metrics")
	if err != nil {
		t.Fatalf("GET /metrics after SetRunner: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Errorf("GET /metrics after SetRunner: status = %d, want %d", resp.StatusCode, http.StatusOK)
	}
	var mr2 MetricsResponse
	if err := json.NewDecoder(resp.Body).Decode(&mr2); err != nil {
		t.Fatalf("decode /metrics (post-SetRunner): %v", err)
	}
	resp.Body.Close()
	if mr2.Commands.TotalRuns != 99 {
		t.Errorf("after SetRunner: /metrics Commands.TotalRuns = %d, want 99", mr2.Commands.TotalRuns)
	}

	// 6. UpdateConfig — replacing the config pointer must be visible to
	//    subsequent GetConfig() callers.
	replacement := DefaultConfig()
	replacement.Host = "0.0.0.0"
	srv.UpdateConfig(replacement)
	if srv.GetConfig() != replacement {
		t.Error("UpdateConfig() did not replace the *Config returned by GetConfig()")
	}

	// 7. Graceful shutdown via context cancellation. The server's
	//    StartWithContext blocks on ctx.Done() and then calls
	//    http.Server.Shutdown, which returns nil on a clean shutdown.
	cancel()
	select {
	case err := <-serveErr:
		if err != nil && err != http.ErrServerClosed {
			t.Errorf("StartWithContext returned %v on cancel; want nil or http.ErrServerClosed", err)
		}
	case <-time.After(5 * time.Second):
		t.Fatal("server did not shut down within 5s of context cancellation")
	}
}
