pub mod config;
pub mod lang;
pub mod runtime;
pub mod engine;
pub mod eval;
#[cfg(feature = "repl")]
pub mod repl;
pub mod log;
pub mod telemetry;

#[cfg(test)]
mod thread_safe_tests {
    use crate::lang::ast::*;
    use crate::runtime::compact::*;
    use crate::engine::*;
    use crate::eval::*;
    use crate::runtime::executor::*;
    use crate::runtime::scope::*;
    use crate::lang::token::*;
    use crate::runtime::value::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn token_types() {
        assert_send::<TokenKind>();
        assert_sync::<TokenKind>();
        assert_send::<Token>();
        assert_sync::<Token>();
    }

    #[test]
    fn value_type() {
        assert_send::<Value>();
        assert_sync::<Value>();
    }

    #[test]
    fn ast_types() {
        assert_send::<Pos>();
        assert_sync::<Pos>();
        assert_send::<Node>();
        assert_sync::<Node>();
        assert_send::<Script>();
        assert_sync::<Script>();
        assert_send::<ShebangDecl>();
        assert_sync::<ShebangDecl>();
        assert_send::<CompactConfig>();
        assert_sync::<CompactConfig>();
        assert_send::<InterpSpan>();
        assert_sync::<InterpSpan>();
        assert_send::<InterpType>();
        assert_sync::<InterpType>();
    }

    #[test]
    fn scope_types() {
        assert_send::<Scope>();
        assert_sync::<Scope>();
        assert_send::<ScopeRef>();
        assert_sync::<ScopeRef>();
    }

    #[test]
    fn compact_types() {
        assert_send::<Config>();
        assert_sync::<Config>();
        assert_send::<Directive>();
        assert_sync::<Directive>();
    }

    #[test]
    fn eval_types() {
        assert_send::<Evaluator>();
        assert_sync::<Evaluator>();
        assert_send::<SharedWriter>();
        assert_sync::<SharedWriter>();
    }

    #[test]
    fn executor_types() {
        assert_send::<Executor>();
        assert_sync::<Executor>();
        assert_send::<ExecResult>();
        assert_sync::<ExecResult>();
    }

    #[test]
    fn engine_types() {
        assert_send::<ExecuteRequest>();
        assert_sync::<ExecuteRequest>();
        assert_send::<ExecuteResponse>();
        assert_sync::<ExecuteResponse>();
    }

    #[test]
    fn telemetry_types() {
        use crate::telemetry::context::SpanContext;
        use crate::telemetry::event::TelemetryEvent;
        use crate::telemetry::filter::Filter;
        assert_send::<SpanContext>();
        assert_sync::<SpanContext>();
        assert_send::<TelemetryEvent>();
        assert_sync::<TelemetryEvent>();
        assert_send::<Filter>();
        assert_sync::<Filter>();
    }
}
