#!/bin/bash
# validate-links.sh - Validate documentation links in Toka workspace

echo "🔍 Validating documentation links..."

broken_links=0

echo "📄 Checking files we created/fixed..."

# Check CONTRIBUTING.md
if [ -f "CONTRIBUTING.md" ]; then
    echo "✅ CONTRIBUTING.md exists"
else
    echo "❌ CONTRIBUTING.md missing"
    ((broken_links++))
fi

# Check TESTS.md
if [ -f "docs/TESTS.md" ]; then
    echo "✅ docs/TESTS.md exists"
else
    echo "❌ docs/TESTS.md missing"
    ((broken_links++))
fi

# Check security directory
if [ -d "docs/security" ] && [ -f "docs/security/README.md" ]; then
    echo "✅ docs/security/ directory and README exist"
else
    echo "❌ docs/security/ directory or README missing"
    ((broken_links++))
fi

# Check testing directory
if [ -d "docs/testing" ] && [ -f "docs/testing/README.md" ]; then
    echo "✅ docs/testing/ directory and README exist"
else
    echo "❌ docs/testing/ directory or README missing"
    ((broken_links++))
fi

# Check troubleshooting directory
if [ -d "docs/troubleshooting" ] && [ -f "docs/troubleshooting/README.md" ]; then
    echo "✅ docs/troubleshooting/ directory and README exist"
else
    echo "❌ docs/troubleshooting/ directory or README missing"
    ((broken_links++))
fi

# Check agent documentation
if [ -d "agents/v0.3.0" ] && [ -f "agents/v0.3.0/README.md" ]; then
    echo "✅ agents/v0.3.0/ directory and README exist"
else
    echo "❌ agents/v0.3.0/ directory or README missing"
    ((broken_links++))
fi

echo ""
echo "📊 Validation Summary:"
echo "   Issues found: $broken_links"

if [ "$broken_links" -eq 0 ]; then
    echo "✅ All critical documentation links are working!"
else
    echo "❌ Found $broken_links issues that need attention"
fi

exit $broken_links 