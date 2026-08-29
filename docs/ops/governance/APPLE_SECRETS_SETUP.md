# Apple Signing Secrets - Configuration Guide
# ============================================================
# T-940: Apple Developer account secrets for GitHub Actions
#
# Add these as Repository Secrets in GitHub:
#   Settings -> Secrets and variables -> Actions -> New repository secret
#
# All values are Base64-encoded where noted.
# ============================================================

## 1. APPLE_CERTIFICATE (required)
##    The .p12 certificate file exported from Keychain Access.
##
##    How to export:
##      1. Open Keychain Access
##      2. Find your "Developer ID Application" certificate
##      3. Right-click -> Export -> Personal Information Exchange (.p12)
##      4. Set a strong password (you'll need it for APPLE_CERTIFICATE_PWD)
##      5. Base64-encode the .p12 file:
##           base64 -i certificate.p12 | pbcopy
##      6. Paste the base64 string as the secret value
##
##    Secret name: APPLE_CERTIFICATE
##    Example: MIIJngIBAzCCCW...

## 2. APPLE_CERTIFICATE_PWD (required)
##    The password you set when exporting the .p12 file above.
##
##    Secret name: APPLE_CERTIFICATE_PWD
##    Example: MyStr0ngP@ssw0rd

## 3. APPLE_ID (required)
##    Your Apple Developer account email address.
##    This is used by notarytool to authenticate with Apple's servers.
##
##    Secret name: APPLE_ID
##    Example: kooshapari@gmail.com

## 4. APPLE_TEAM_ID (required)
##    Your Apple Developer Team ID.
##    Found at: https://developer.apple.com/account -> Membership details
##    It's a 10-character alphanumeric string (e.g., "AB12CD34EF").
##
##    Secret name: APPLE_TEAM_ID
##    Example: AB12CD34EF

## 5. APPLE_APP_PASSWORD (required)
##    An app-specific password generated for notarytool.
##    Do NOT use your Apple ID password directly.
##
##    How to generate:
##      1. Go to https://appleid.apple.com
##      2. Sign in with your Apple ID
##      3. Navigate to "App-Specific Passwords" under Security
##      4. Click "Generate an app-specific password"
##      5. Label it "GitHub Actions notarytool"
##      6. Copy the generated password
##
##    Secret name: APPLE_APP_PASSWORD
##    Example: abcd-efgh-ijkl-mnop

## ============================================================
## Quick Setup Script (run once on macOS)
## ============================================================
##
## # Export the certificate from Keychain
## security export -k ~/Library/Keychains/login.keychain-db \
##   -t certs -p -o cert.der
## base64 -i cert.der > cert.b64
##
## # Or if you have the .p12 already:
## base64 -i certificate.p12 > cert.b64
##
## # Then use `cat cert.b64` output as APPLE_CERTIFICATE secret value
##
## ============================================================
## Verification (after setup)
## ============================================================
##
## After adding all 5 secrets, trigger a manual run:
##   gh workflow run codesign.yml --ref main
##
## The macOS-sign job should:
##   1. Import the certificate successfully
##   2. Build the release binary
##   3. Codesign with "-" (ad-hoc) for now, or with your identity
##   4. Submit to Apple notary service
##   5. Wait for notarization (usually 1-5 minutes)
##   6. Staple the notarization ticket
##   7. Verify the codesign
## ============================================================
