#!/bin/bash
# TwoDo Cloudflare Infrastructure Setup
# For Wrangler 4.x (updated syntax)

echo "🚀 Setting up TwoDo on Cloudflare..."

# 1. Create D1 database
echo "📊 Creating D1 database..."
wrangler d1 create twodo-production

# 2. Create KV namespace (using modern syntax)
echo "🗄️ Creating KV namespace..."
wrangler kv namespace create "twodo-cache"

# 3. Create R2 bucket for file storage (requires R2 to be enabled first)
echo "📁 Creating R2 bucket..."
echo "⚠️  Make sure R2 is enabled in your Cloudflare dashboard first!"
wrangler r2 bucket create twodo-files

echo "✅ Infrastructure setup complete!"
echo ""
echo "📝 Next steps:"
echo "1. Get your resource IDs:"
echo "   wrangler d1 list"
echo "   wrangler kv namespace list"
echo "2. Copy wrangler.example.toml to wrangler.toml"
echo "3. Update wrangler.toml with YOUR actual resource IDs"
echo "4. Deploy the database schema"
echo "5. Set your secrets securely"
echo "6. Deploy your Workers!"