use crate::errors::AgentError;
use hubbub_domain::entities::agent::AgentBudget;
use hubbub_domain::ports::llm::LlmUsage;
use std::time::Instant;

pub struct BudgetTracker {
    budget: AgentBudget,
    steps_taken: u32,
    tokens_used: u64,
    cost_usd: f64,
    start_time: Instant,
}

impl BudgetTracker {
    pub fn new(budget: AgentBudget) -> Self {
        Self {
            budget,
            steps_taken: 0,
            tokens_used: 0,
            cost_usd: 0.0,
            start_time: Instant::now(),
        }
    }

    pub fn check_limits(&self) -> Result<(), AgentError> {
        if self.budget.timeout_s > 0 && self.start_time.elapsed().as_secs() >= self.budget.timeout_s
        {
            return Err(AgentError::BudgetExceeded(format!(
                "Timeout exceeded: {}s >= limit {}s",
                self.start_time.elapsed().as_secs(),
                self.budget.timeout_s
            )));
        }

        if self.budget.max_steps > 0 && self.steps_taken >= self.budget.max_steps {
            return Err(AgentError::BudgetExceeded(format!(
                "Max steps exceeded: {} >= limit {}",
                self.steps_taken, self.budget.max_steps
            )));
        }

        self.check_token_limit()
    }

    /// Token check alone, for after a step finished (the step limit is about starting the next one).
    pub fn check_token_limit(&self) -> Result<(), AgentError> {
        if self.budget.max_tokens > 0 && self.tokens_used >= self.budget.max_tokens {
            return Err(AgentError::BudgetExceeded(format!(
                "Max tokens exceeded: {} >= limit {}",
                self.tokens_used, self.budget.max_tokens
            )));
        }

        Ok(())
    }

    pub fn record_step(&mut self) -> Result<(), AgentError> {
        self.check_limits()?;
        self.steps_taken += 1;
        Ok(())
    }

    /// Usage arrives mid-stream; it is only recorded here and enforced between steps,
    /// so an answer that was already streamed is never thrown away.
    pub fn record_usage(&mut self, usage: &LlmUsage) {
        self.tokens_used += usage.prompt_tokens + usage.completion_tokens;
    }

    pub fn steps_taken(&self) -> u32 {
        self.steps_taken
    }

    pub fn tokens_used(&self) -> u64 {
        self.tokens_used
    }

    pub fn cost_usd(&self) -> f64 {
        self.cost_usd
    }
}
