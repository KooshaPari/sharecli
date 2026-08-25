#!/usr/bin/env python3
"""
88-Pillar Scorecard CI Script v2
Audits a repository against 88 quality pillars.
Supports --exclude for enterprise-only pillars that don't apply to library/CLI repos.
"""
import os, sys, json, argparse
from pathlib import Path

PILLARS = [
    {"id":1,"name":"README","check":lambda p:(p/"README.md").exists() or (p/"readme.md").exists()},
    {"id":2,"name":"LICENSE","check":lambda p:any(p.glob("LICENSE*"))},
    {"id":3,"name":"CONTRIBUTING","check":lambda p:(p/"CONTRIBUTING.md").exists()},
    {"id":4,"name":"CODE_OF_CONDUCT","check":lambda p:(p/"CODE_OF_CONDUCT.md").exists()},
    {"id":5,"name":"SECURITY","check":lambda p:(p/"SECURITY.md").exists()},
    {"id":6,"name":"CHANGELOG","check":lambda p:any(p.glob("CHANGELOG*")) or any(p.glob("CHANGES*"))},
    {"id":7,"name":"CLAUDE_MD","check":lambda p:(p/"CLAUDE.md").exists()},
    {"id":8,"name":"EDITORCONFIG","check":lambda p:(p/".editorconfig").exists()},
    {"id":9,"name":"GITIGNORE","check":lambda p:(p/".gitignore").exists()},
    {"id":10,"name":"DOCKERFILE","check":lambda p:(p/"Dockerfile").exists()},
    {"id":11,"name":"DOCKER_COMPOSE","check":lambda p:(p/"docker-compose.yml").exists() or (p/"docker-compose.yaml").exists()},
    {"id":12,"name":"MAKEFILE","check":lambda p:(p/"Makefile").exists()},
    {"id":13,"name":"JUSTFILE","check":lambda p:(p/"Justfile").exists()},
    {"id":14,"name":"PACKAGE_JSON","check":lambda p:(p/"package.json").exists() or any(p.glob("**/package.json"))},
    {"id":15,"name":"PYPROJECT_TOML","check":lambda p:(p/"pyproject.toml").exists()},
    {"id":16,"name":"CARGO_TOML","check":lambda p:(p/"Cargo.toml").exists() or any(p.glob("**/Cargo.toml"))},
    {"id":17,"name":"GO_MOD","check":lambda p:(p/"go.mod").exists()},
    {"id":18,"name":"ENV_EXAMPLE","check":lambda p:(p/".env.example").exists() or (p/".env.template").exists()},
    {"id":19,"name":"CI_WORKFLOW","check":lambda p:len(list((p/".github/workflows").glob("*.yml")))>0 if (p/".github/workflows").exists() else False},
    {"id":20,"name":"CODEOWNERS","check":lambda p:(p/".github/CODEOWNERS").exists() or (p/"CODEOWNERS").exists()},
    {"id":21,"name":"DEPENDABOT","check":lambda p:(p/".github/dependabot.yml").exists() or (p/".github/dependabot.yaml").exists()},
    {"id":22,"name":"ISSUE_TEMPLATE","check":lambda p:(p/".github/ISSUE_TEMPLATE").exists() and any((p/".github/ISSUE_TEMPLATE").iterdir()) if (p/".github/ISSUE_TEMPLATE").exists() else False},
    {"id":23,"name":"PR_TEMPLATE","check":lambda p:any(p.glob(".github/PULL_REQUEST_TEMPLATE*")) or any(p.glob(".github/pull_request_template*"))},
    {"id":24,"name":"FUZZ_TESTS","check":lambda p:(p/"fuzz").exists() or any(p.glob("**/fuzz_*.rs"))},
    {"id":25,"name":"BENCHMARKS","check":lambda p:(p/"benches").exists() or (p/"bench").exists()},
    {"id":26,"name":"MUTANT_TESTS","check":lambda p:(p/"mutants.out").exists() or (p/"mutants.toml").exists()},
    {"id":27,"name":"UNIT_TESTS","check":lambda p:any(p.glob("**/test_*.py")) or any(p.glob("**/*_test.rs")) or any(p.glob("**/*.test.ts")) or any(p.glob("**/*.spec.ts"))},
    {"id":28,"name":"INTEGRATION_TESTS","check":lambda p:(p/"tests"/"integration").exists() or (p/"test"/"integration").exists()},
    {"id":29,"name":"E2E_TESTS","check":lambda p:(p/"tests"/"e2e").exists() or (p/"e2e").exists()},
    {"id":30,"name":"CODE_COVERAGE","check":lambda p:(p/".coveragerc").exists() or (p/"codecov.yml").exists()},
    {"id":31,"name":"LINTING","check":lambda p:(p/".eslintrc.js").exists() or (p/"ruff.toml").exists() or (p/".clippy.toml").exists() or (p/"clippy.toml").exists()},
    {"id":32,"name":"FORMATTING","check":lambda p:(p/".prettierrc").exists() or (p/"rustfmt.toml").exists()},
    {"id":33,"name":"SECURITY_SCANNING","check":lambda p:(p/".github/workflows"/"codeql.yml").exists() or (p/".github/workflows"/"security.yml").exists() or (p/".github/workflows"/"trivy.yml").exists() or any((p/".github/workflows").glob("*secur*.yml")) if (p/".github/workflows").exists() else False},
    {"id":34,"name":"DEPENDENCY_AUDIT","check":lambda p:(p/".snyk").exists() or (p/"audit-ci.json").exists() or (p/"deny.toml").exists()},
    {"id":35,"name":"OPENAPI_SPEC","check":lambda p:any(p.glob("**/openapi.json")) or any(p.glob("**/openapi.yaml")) or any(p.glob("**/swagger.json"))},
    {"id":36,"name":"DOCS_SITE","check":lambda p:(p/"docs").exists() or (p/"website").exists()},
    {"id":37,"name":"I18N","check":lambda p:(p/"locales").exists() or (p/"i18n").exists()},
    {"id":38,"name":"A11Y","check":lambda p:any(p.glob("**/*a11y*"))},
    {"id":39,"name":"LOAD_TESTING","check":lambda p:(p/"loadtests").exists() or (p/"load_tests").exists()},
    {"id":40,"name":"CONTAINER_SCANNING","check":lambda p:any(p.glob("**/trivyignore"))},
    {"id":41,"name":"FEATURE_FLAGS","check":lambda p:any(p.glob("**/*feature*flag*"))},
    {"id":42,"name":"LOGGING","check":lambda p:any(p.glob("**/logging.py")) or any(p.glob("**/*logger*")) or any(p.glob("**/*tracing*")) or any(p.glob("**/*log*.toml"))},
    {"id":43,"name":"MONITORING","check":lambda p:(p/"monitoring").exists() or (p/"prometheus.yml").exists()},
    {"id":44,"name":"TRACING","check":lambda p:any(p.glob("**/*opentelemetry*")) or any(p.glob("**/*tracing*"))},
    {"id":45,"name":"ALERTING","check":lambda p:(p/"alerts.yml").exists() or (p/"alerting_rules.yml").exists()},
    {"id":46,"name":"RATE_LIMITING","check":lambda p:any(p.glob("**/*rate*limit*"))},
    {"id":47,"name":"CACHING","check":lambda p:any(p.glob("**/*cache*config*"))},
    {"id":48,"name":"SSL_TLS","check":lambda p:any(p.glob("**/ssl*.conf"))},
    {"id":49,"name":"WAF","check":lambda p:any(p.glob("**/*waf*"))},
    {"id":50,"name":"MFA","check":lambda p:any(p.glob("**/*mfa*"))},
    {"id":51,"name":"RBAC","check":lambda p:any(p.glob("**/*rbac*"))},
    {"id":52,"name":"AUDIT_LOGS","check":lambda p:any(p.glob("**/*audit*log*"))},
    {"id":53,"name":"DATABASE_MIGRATIONS","check":lambda p:(p/"migrations").exists() or (p/"migrate").exists()},
    {"id":54,"name":"ENV_VARS","check":lambda p:(p/".env").exists() or (p/".env.local").exists()},
    {"id":55,"name":"KUBERNETES","check":lambda p:any(p.glob("**/k8s/*.yml")) or any(p.glob("**/kubernetes/*.yml"))},
    {"id":56,"name":"HELM","check":lambda p:(p/"Chart.yaml").exists()},
    {"id":57,"name":"TERRAFORM","check":lambda p:any(p.glob("**/*.tf"))},
    {"id":58,"name":"ANSIBLE","check":lambda p:(p/"playbooks").exists() or (p/"roles").exists()},
    {"id":59,"name":"CLOUDFORMATION","check":lambda p:any(p.glob("**/*.template"))},
    {"id":60,"name":"CANARY_DEPLOY","check":lambda p:any(p.glob("**/*canary*"))},
    {"id":61,"name":"ROLLBACK","check":lambda p:any(p.glob("**/*rollback*"))},
    {"id":62,"name":"DATA_PRIVACY","check":lambda p:any(p.glob("**/*privacy*")) or any(p.glob("**/*gdpr*"))},
    {"id":63,"name":"COMPLIANCE","check":lambda p:any(p.glob("**/*compliance*"))},
    {"id":64,"name":"LICENSE_SCANNING","check":lambda p:(p/"license-checker.json").exists()},
    {"id":65,"name":"SECRET_SCANNING","check":lambda p:(p/".gitleaks.toml").exists() or (p/"gitleaks.toml").exists()},
    {"id":66,"name":"IAAC","check":lambda p:any(p.glob("**/terraform/*.tf")) or any(p.glob("**/ansible/*.yml"))},
    {"id":67,"name":"CDN","check":lambda p:any(p.glob("**/*cdn*"))},
    {"id":68,"name":"FIREWALL","check":lambda p:any(p.glob("**/*firewall*"))},
    {"id":69,"name":"VPN","check":lambda p:any(p.glob("**/*vpn*"))},
    {"id":70,"name":"SSO","check":lambda p:any(p.glob("**/*sso*")) or any(p.glob("**/*oauth*"))},
    {"id":71,"name":"BACKUPS","check":lambda p:any(p.glob("**/*backup*")) or any(p.glob("**/*restore*"))},
    {"id":72,"name":"DISASTER_RECOVERY","check":lambda p:any(p.glob("**/*disaster*recovery*"))},
    {"id":73,"name":"STRESS_TESTING","check":lambda p:any(p.glob("**/*stress*test*"))},
    {"id":74,"name":"PERFORMANCE_TESTING","check":lambda p:any(p.glob("**/*perf*test*"))},
    {"id":75,"name":"SEO","check":lambda p:any(p.glob("**/*seo*")) or (p/"sitemap.xml").exists()},
    {"id":76,"name":"ANALYTICS","check":lambda p:any(p.glob("**/*analytics*")) or any(p.glob("**/*gtag*"))},
    {"id":77,"name":"FEEDBACK","check":lambda p:any(p.glob("**/*feedback*"))},
    {"id":78,"name":"SUPPORT","check":lambda p:(p/"SUPPORT.md").exists() or any(p.glob("**/*support*"))},
    {"id":79,"name":"ROADMAP","check":lambda p:(p/"ROADMAP.md").exists() or any(p.glob("**/*roadmap*"))},
    {"id":80,"name":"STATUS_PAGE","check":lambda p:any(p.glob("**/*status*page*"))},
    {"id":81,"name":"INCIDENT_RESPONSE","check":lambda p:any(p.glob("**/*incident*response*"))},
    {"id":82,"name":"DATA_SEEDING","check":lambda p:(p/"seeds").exists() or (p/"seed").exists()},
    {"id":83,"name":"DATA_CLEANUP","check":lambda p:any(p.glob("**/*cleanup*")) or any(p.glob("**/*prune*"))},
    {"id":84,"name":"THROTTLING","check":lambda p:any(p.glob("**/*throttl*"))},
    {"id":85,"name":"BUSINESS_CONTINUITY","check":lambda p:any(p.glob("**/*business*continuity*"))},
    {"id":86,"name":"SUCCESSION_PLANNING","check":lambda p:any(p.glob("**/*succession*"))},
    {"id":87,"name":"SHIPPING","check":lambda p:(p/".releaserc").exists() or (p/"release.config.js").exists()},
    {"id":88,"name":"RELEASE_NOTES","check":lambda p:any(p.glob("**/*release*note*"))},
]

# Enterprise-only pillars that don't apply to library/CLI/registry repos
ENTERPRISE_EXCLUDE = {
    # Enterprise-only infrastructure
    "SSL_TLS","WAF","MFA","RBAC","AUDIT_LOGS","KUBERNETES","HELM",
    "TERRAFORM","ANSIBLE","CLOUDFORMATION","CANARY_DEPLOY","ROLLBACK",
    "DATA_PRIVACY","COMPLIANCE","IAAC","CDN","FIREWALL","VPN",
    "SSO","BACKUPS","DISASTER_RECOVERY","STRESS_TESTING",
    "PERFORMANCE_TESTING","SEO","ANALYTICS","FEEDBACK","SUPPORT","ROADMAP",
    "STATUS_PAGE","INCIDENT_RESPONSE","DATA_SEEDING","DATA_CLEANUP",
    "THROTTLING","BUSINESS_CONTINUITY","SUCCESSION_PLANNING","SHIPPING","RELEASE_NOTES",
    "LICENSE_SCANNING",
    # Non-applicable for library/CLI/registry repos
    "GO_MOD", "PYPROJECT_TOML", "DATABASE_MIGRATIONS",
    "I18N", "A11Y", "LOAD_TESTING", "CONTAINER_SCANNING",
    "OPENAPI_SPEC", "CACHING", "RATE_LIMITING", "FEATURE_FLAGS",
    "TRACING", "ALERTING", "MONITORING",
}

def audit_repo(repo_path, exclude=None):
    path = Path(repo_path)
    if not path.is_dir():
        raise ValueError(f"Path {repo_path} is not a directory")
    excluded = set(exclude) if exclude else set()
    results, score, skipped = [], 0, 0
    for pillar in PILLARS:
        if pillar["name"] in excluded:
            results.append({"id":pillar["id"],"name":pillar["name"],"passed":None,"excluded":True})
            skipped += 1
            continue
        try:
            passed = pillar["check"](path)
            if isinstance(passed, list): passed = len(passed) > 0
            results.append({"id":pillar["id"],"name":pillar["name"],"passed":bool(passed)})
            if passed: score += 1
        except Exception as e:
            results.append({"id":pillar["id"],"name":pillar["name"],"passed":False,"error":str(e)})
    active = len(PILLARS) - skipped
    return {
        "score": score,
        "total": active,
        "original_total": len(PILLARS),
        "skipped": skipped,
        "percentage": (score/active*100) if active else 0,
        "results": results
    }

def main():
    parser = argparse.ArgumentParser(description="88-Pillar Scorecard Audit v2")
    parser.add_argument("path", help="Path to repository")
    parser.add_argument("--threshold", type=int, default=45,
                        help="Minimum score to pass (default: 45 out of active pillars)")
    parser.add_argument("--output", choices=["text","json","markdown"], default="text")
    parser.add_argument("--fail-on-drop", action="store_true")
    parser.add_argument("--exclude", nargs="*", default=[],
                        help="Pillar names to exclude (e.g. SSL_TLS WAF MFA)")
    parser.add_argument("--exclude-enterprise", action="store_true",
                        help="Exclude all enterprise-only pillars (38 pillars)")
    args = parser.parse_args()

    exclude = list(args.exclude)
    if args.exclude_enterprise:
        exclude.extend(ENTERPRISE_EXCLUDE)

    try:
        report = audit_repo(args.path, exclude=exclude)
        if args.output == "json":
            print(json.dumps(report, indent=2))
        elif args.output == "markdown":
            print("# 88-Pillar Scorecard Report (v2)\n")
            print(f"**Score:** {report['score']}/{report['total']} ({report['percentage']:.1f}%)\n")
            if report['skipped']:
                print(f"**Skipped:** {report['skipped']} enterprise pillars (excluded)\n")
            print(f"**Threshold:** {args.threshold}\n")
            print(f"**Status:** {'PASS' if report['score'] >= args.threshold else 'FAIL'}\n")
            print("## Results\n| ID | Pillar | Status |\n|---|--------|--------|")
            for r in report["results"]:
                if r.get("excluded"):
                    print(f"| {r['id']} | {r['name']} | SKIP (enterprise) |")
                else:
                    print(f"| {r['id']} | {r['name']} | {'PASS' if r['passed'] else 'FAIL'} |")
        else:
            print(f"Scorecard v2: {report['score']}/{report['total']} ({report['percentage']:.1f}%)")
            if report['skipped']:
                print(f"Excluded: {report['skipped']} enterprise pillars")
            print(f"Threshold: {args.threshold}")
            if report['score'] >= args.threshold:
                print("Status: PASS")
            else:
                failed = [r['name'] for r in report['results'] if not r.get('excluded') and not r['passed']]
                print(f"Status: FAIL\nFailed: {', '.join(failed)}")
        if args.fail_on_drop and report['score'] < args.threshold:
            sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(2)

if __name__ == "__main__":
    main()
