// High-performance Cloudflare Workers implementation
use worker::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Optimized data structures for minimal memory usage
#[derive(Serialize, Deserialize, Clone)]
pub struct OptimizedExpense {
    pub id: String,
    pub desc: String,         // Shortened field names for smaller JSON
    pub amt: f64,
    pub dt: i64,              // Unix timestamp
    pub by: String,           // paid_by user_id
    pub grp: String,          // group_id
    pub typ: u8,              // split_type as enum index
    pub spl: Vec<Split>,      // splits
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Split {
    pub u: String,            // user_id
    pub a: f64,               // amount
    pub s: bool,              // is_settled
}

// Cached balance computation (stored in KV for 1 hour)
#[derive(Serialize, Deserialize)]
pub struct BalanceCache {
    pub balances: HashMap<String, f64>,
    pub computed_at: i64,
    pub ttl: i64,
}

// Performance-optimized expense service
pub struct CloudflareExpenseService {
    db: D1Database,
    kv: KvStore,
}

impl CloudflareExpenseService {
    pub fn new(db: D1Database, kv: KvStore) -> Self {
        Self { db, kv }
    }

    // Optimized balance calculation with caching
    pub async fn get_group_balances(&self, group_id: &str) -> Result<HashMap<String, f64>, Error> {
        let cache_key = format!("balances:{}", group_id);
        
        // Try cache first (sub-millisecond response)
        if let Ok(Some(cached)) = self.kv.get(&cache_key).json::<BalanceCache>().await {
            if cached.computed_at + cached.ttl > js_sys::Date::now() as i64 {
                return Ok(cached.balances);
            }
        }

        // Compute balances with optimized SQL
        let balances = self.compute_balances_optimized(group_id).await?;
        
        // Cache for 1 hour
        let cache = BalanceCache {
            balances: balances.clone(),
            computed_at: js_sys::Date::now() as i64,
            ttl: 3600_000, // 1 hour in milliseconds
        };
        
        // Fire-and-forget cache update
        let _ = self.kv.put(&cache_key, &cache)?.expiration_ttl(3600).execute().await;
        
        Ok(balances)
    }

    // Single optimized query instead of multiple round trips
    async fn compute_balances_optimized(&self, group_id: &str) -> Result<HashMap<String, f64>, Error> {
        let stmt = self.db.prepare("
            SELECT user_id, total_paid, total_owed, net_balance 
            FROM expense_balances 
            WHERE group_id = ?
        ");
        
        let result = stmt.bind(&[group_id.into()])?.all().await?;
        
        let mut balances = HashMap::new();
        for row in result.results()? {
            let user_id: String = row.get("user_id")?;
            let net_balance: f64 = row.get("net_balance")?;
            balances.insert(user_id, net_balance);
        }
        
        Ok(balances)
    }

    // Optimized expense creation with batch operations
    pub async fn create_expense_optimized(
        &self,
        expense: &OptimizedExpense,
    ) -> Result<(), Error> {
        // Use transaction for consistency
        let tx = self.db.transaction().await?;
        
        // Insert expense
        let expense_stmt = tx.prepare("
            INSERT INTO expenses (id, description, amount, date, paid_by, group_id, split_type)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        ");
        
        expense_stmt.bind(&[
            expense.id.clone().into(),
            expense.desc.clone().into(),
            expense.amt.into(),
            expense.dt.into(),
            expense.by.clone().into(),
            expense.grp.clone().into(),
            expense.typ.into(),
        ])?.run().await?;

        // Batch insert splits
        let split_stmt = tx.prepare("
            INSERT INTO expense_splits (expense_id, user_id, amount, is_settled)
            VALUES (?, ?, ?, ?)
        ");
        
        for split in &expense.spl {
            split_stmt.bind(&[
                expense.id.clone().into(),
                split.u.clone().into(),
                split.a.into(),
                split.s.into(),
            ])?.run().await?;
        }

        tx.commit().await?;
        
        // Invalidate cache
        let cache_key = format!("balances:{}", expense.grp);
        let _ = self.kv.delete(&cache_key).await;
        
        Ok(())
    }
}

// Optimized HTTP handlers with minimal allocations
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    // Enable panic hooks for debugging
    console_error_panic_hook::set_once();
    
    // Fast route matching without regex
    let url = req.url()?;
    let path = url.path();
    
    match (req.method(), path) {
        (Method::Get, path) if path.starts_with("/api/expenses/balances/") => {
            let group_id = path.strip_prefix("/api/expenses/balances/").unwrap();
            handle_get_balances(req, env, group_id).await
        }
        (Method::Post, "/api/expenses") => {
            handle_create_expense(req, env).await
        }
        _ => Response::error("Not Found", 404),
    }
}

// Optimized balance handler
async fn handle_get_balances(
    _req: Request,
    env: Env,
    group_id: &str,
) -> Result<Response> {
    let db = env.d1("DB")?;
    let kv = env.kv("KV")?;
    
    let service = CloudflareExpenseService::new(db, kv);
    let balances = service.get_group_balances(group_id).await?;
    
    // Minimal JSON response
    Response::from_json(&balances)
}

// Optimized expense creation
async fn handle_create_expense(
    mut req: Request,
    env: Env,
) -> Result<Response> {
    let expense: OptimizedExpense = req.json().await?;
    
    let db = env.d1("DB")?;
    let kv = env.kv("KV")?;
    
    let service = CloudflareExpenseService::new(db, kv);
    service.create_expense_optimized(&expense).await?;
    
    Response::ok("Created")
}
