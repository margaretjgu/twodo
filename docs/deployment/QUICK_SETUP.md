# ⚡ TwoDo Cloudflare Quick Setup

## ✅ Resources Created

Your Cloudflare infrastructure is ready! Here's what you should have:

- ✅ **KV Namespace**: `twodo-cache` (get ID with: `wrangler kv namespace list`)
- ✅ **D1 Database**: `twodo-production` (get ID with: `wrangler d1 list`)
- ⏳ **R2 Storage**: Needs to be enabled (see step 1 below)

## 🚀 Next Steps (5 minutes total)

### Step 1: Enable R2 Storage (1 minute)
1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com)
2. Navigate to **R2 Object Storage** in the sidebar
3. Click **"Purchase R2 Plan"** (select the **FREE tier** - 10GB free)
4. Once enabled, run: `wrangler r2 bucket create twodo-files`

### Step 2: Update Your Configuration
```bash
# Get your resource IDs
wrangler d1 list          # Copy your database ID
wrangler kv namespace list # Copy your KV namespace ID

# Update wrangler.toml with YOUR actual IDs
cp wrangler.example.toml wrangler.toml
# Edit wrangler.toml with your actual resource IDs
```

### Step 3: Set Secrets (1 minute) - SECURE
```bash
# Set your JWT secret (make it long and random)
wrangler secret put JWT_SECRET
# Enter: a-super-secure-random-string-min-32-characters

# Set FCM key for push notifications (get from Firebase Console)
wrangler secret put FCM_SERVER_KEY
# Enter: your-firebase-server-key
```

### Step 4: Deploy Your API (30 seconds)
```bash
# Deploy to Cloudflare Workers (global edge network)
wrangler deploy
```

## 🎯 Expected Result

After deployment, you'll have:
- **Global API** running at: `https://twodo-api.your-subdomain.workers.dev`
- **Sub-100ms responses** worldwide
- **$0/month cost** for your usage
- **99.9% uptime** with DDoS protection
- **Push notifications** ready to go

## 📱 Firebase Setup (Optional - for push notifications)

If you want push notifications:
1. Go to [Firebase Console](https://console.firebase.google.com)
2. Create new project: "TwoDo"
3. Add Android/iOS apps
4. Get the **Server Key** from Cloud Messaging settings
5. Use it in Step 3 above

## 💡 Pro Tips

### Cost Optimization
- Your current setup will cost **$0/month**
- Only pay when you scale beyond free tiers
- R2 gives you 10GB free storage
- KV gives you 100k operations/day free

### Security Best Practices
- ✅ Never commit `wrangler.toml` with real IDs
- ✅ Use `wrangler secret put` for all sensitive data
- ✅ Keep your `.gitignore` up to date
- ✅ Use example/template files for documentation