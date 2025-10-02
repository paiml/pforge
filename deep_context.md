# Project Context

**Language**: rust
**Project Path**: .

## Project Structure

- **Total Files**: 36
- **Total Functions**: 45
- **Median Cyclomatic**: 1.00
- **Median Cognitive**: 0.00

## Quality Scorecard

- **Overall Health**: 83.3%
- **Maintainability Index**: 70.0
- **Complexity Score**: 95.0
- **Test Coverage**: 65.0%

## Files

### ./crates/pforge-cli/src/commands/build.rs

**File Complexity**: 4 | **Functions**: 1

- **Function**: `execute` [complexity: 4] [cognitive: 3] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-cli/src/commands/dev.rs

**File Complexity**: 1 | **Functions**: 1

- **Function**: `execute` [complexity: 1] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 1 items] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-cli/src/commands/mod.rs

**File Complexity**: 1 | **Functions**: 0


### ./crates/pforge-cli/src/commands/new.rs

**File Complexity**: 9 | **Functions**: 1

- **Function**: `execute` [complexity: 9] [cognitive: 8] [big-o: O(n log n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-cli/src/commands/serve.rs

**File Complexity**: 3 | **Functions**: 1

- **Function**: `execute` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]

### ./crates/pforge-cli/src/main.rs

**File Complexity**: 6 | **Functions**: 1

- **Struct**: `Cli` [fields: 1]
- **Enum**: `Commands` [variants: 4]
- **Function**: `main` [complexity: 6] [cognitive: 9] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-cli/tests/scaffold_tests.rs

**File Complexity**: 1 | **Functions**: 11

- **Function**: `workspace_root` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_workspace_compiles` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_all_crates_exist` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_pmat_configuration_exists` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_workspace_has_correct_members` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_runtime_crate_has_pmcp_dependency` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_cli_crate_has_clap_dependency` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_codegen_crate_has_syn_quote` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_project_templates_exist` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_pre_commit_hook_exists` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]
- **Function**: `red_test_all_crates_compile_independently` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(2)] [tdg: 2.5]

### ./crates/pforge-codegen/src/generator.rs

**File Complexity**: 2 | **Functions**: 5

- **Enum**: `CodegenError` [variants: 2]
- **Function**: `generate_param_struct` [complexity: 5] [cognitive: 7] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `generate_handler_registration` [complexity: 4] [cognitive: 6] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `to_pascal_case` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `rust_type_from_simple` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `format_string_vec` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-codegen/src/lib.rs

**File Complexity**: 4 | **Functions**: 2

- **Function**: `generate_all` [complexity: 5] [cognitive: 7] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `write_generated_code` [complexity: 3] [cognitive: 2] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-config/src/error.rs

**File Complexity**: 1 | **Functions**: 0

- **Enum**: `ConfigError` [variants: 5]

### ./crates/pforge-config/src/lib.rs

**File Complexity**: 1 | **Functions**: 0


### ./crates/pforge-config/src/parser.rs

**File Complexity**: 1 | **Functions**: 2

- **Function**: `parse_config` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `parse_config_from_str` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-config/src/types.rs

**File Complexity**: 1 | **Functions**: 1

- **Struct**: `ForgeConfig` [fields: 5]
- **Struct**: `ForgeMetadata` [fields: 4]
- **Enum**: `TransportType` [variants: 3]
- **Enum**: `OptimizationLevel` [variants: 2]
- **Enum**: `ToolDef` [variants: 4]
- **Impl**: `impl ToolDef { pub fn name (& self) -> & str { match self { ToolDef :: Native { name , .. } => name , ToolDef :: Cli { name , .. } => name , ToolDef :: Http { name , .. } => name , ToolDef :: Pipeline { name , .. } => name , } } } . self_ty`
- **Struct**: `HandlerRef` [fields: 2]
- **Struct**: `ParamSchema` [fields: 1]
- **Enum**: `ParamType` [variants: 2]
- **Enum**: `SimpleType` [variants: 6]
- **Struct**: `Validation` [fields: 5]
- **Enum**: `HttpMethod` [variants: 5]
- **Enum**: `AuthConfig` [variants: 3]
- **Struct**: `PipelineStep` [fields: 5]
- **Enum**: `ErrorPolicy` [variants: 2]
- **Struct**: `ResourceDef` [fields: 3]
- **Enum**: `ResourceOperation` [variants: 3]
- **Struct**: `PromptDef` [fields: 4]
- **Struct**: `StateDef` [fields: 3]
- **Enum**: `StateBackend` [variants: 2]
- **Function**: `default_transport` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Impl**: `Default` for `impl Default for OptimizationLevel { fn default () -> Self { Self :: Debug } } . self_ty`
- **Impl**: `Default` for `impl Default for ErrorPolicy { fn default () -> Self { Self :: FailFast } } . self_ty`

### ./crates/pforge-config/src/validator.rs

**File Complexity**: 4 | **Functions**: 2

- **Function**: `validate_config` [complexity: 6] [cognitive: 12] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `validate_handler_path` [complexity: 3] [cognitive: 6] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-integration-tests/integration_test.rs

**File Complexity**: 1 | **Functions**: 12

- **Function**: `test_config_parsing_all_tool_types` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_config_with_resources_and_prompts` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_state_management_persistence` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_middleware_chain_with_recovery` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_retry_with_timeout` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_circuit_breaker_integration` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_prompt_manager_full_workflow` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_resource_manager_uri_matching` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_error_tracker_classification` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_forge_metadata_defaults` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_full_middleware_stack` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `test_config_validation_duplicate_tools` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-macro/src/lib.rs

**File Complexity**: 1 | **Functions**: 0


### ./crates/pforge-runtime/src/error.rs

**File Complexity**: 1 | **Functions**: 0

- **Enum**: `Error` [variants: 6]

### ./crates/pforge-runtime/src/handler.rs

**File Complexity**: 1 | **Functions**: 0

- **Trait**: `Handler`

### ./crates/pforge-runtime/src/handlers/cli.rs

**File Complexity**: 1 | **Functions**: 0

- **Struct**: `CliHandler` [fields: 6]
- **Struct**: `CliInput` [fields: 2]
- **Struct**: `CliOutput` [fields: 3]
- **Impl**: `impl CliHandler { pub fn new (command : String , args : Vec < String > , cwd : Option < String > , env : HashMap < String , String > , timeout_ms : Option < u64 > , stream : bool ,) -> Self { Self { command , args , cwd , env , timeout_ms , stream , } } pub async fn execute (& self , input : CliInput) -> Result < CliOutput > { let mut cmd = Command :: new (& self . command) ; cmd . args (& self . args) ; cmd . args (& input . args) ; if let Some (cwd) = & self . cwd { cmd . current_dir (cwd) ; } for (k , v) in & self . env { cmd . env (k , v) ; } for (k , v) in & input . env { cmd . env (k , v) ; } cmd . stdout (Stdio :: piped ()) ; cmd . stderr (Stdio :: piped ()) ; let exec_future = async { let output = cmd . output () . await . map_err (| e | { Error :: Handler (format ! ("Failed to execute command '{}': {}" , self . command , e)) }) ? ; Ok :: < _ , Error > (CliOutput { stdout : String :: from_utf8_lossy (& output . stdout) . to_string () , stderr : String :: from_utf8_lossy (& output . stderr) . to_string () , exit_code : output . status . code () . unwrap_or (- 1) , }) } ; if let Some (timeout_ms) = self . timeout_ms { timeout (Duration :: from_millis (timeout_ms) , exec_future) . await . map_err (| _ | Error :: Timeout) ? } else { exec_future . await } } } . self_ty`

### ./crates/pforge-runtime/src/handlers/http.rs

**File Complexity**: 1 | **Functions**: 0

- **Struct**: `HttpHandler` [fields: 5]
- **Enum**: `HttpMethod` [variants: 5]
- **Enum**: `AuthConfig` [variants: 3]
- **Struct**: `HttpInput` [fields: 2]
- **Struct**: `HttpOutput` [fields: 3]
- **Impl**: `impl HttpHandler { pub fn new (endpoint : String , method : HttpMethod , headers : HashMap < String , String > , auth : Option < AuthConfig > ,) -> Self { Self { endpoint , method , headers , auth , client : Client :: new () , } } pub async fn execute (& self , input : HttpInput) -> Result < HttpOutput > { let method = match self . method { HttpMethod :: Get => Method :: GET , HttpMethod :: Post => Method :: POST , HttpMethod :: Put => Method :: PUT , HttpMethod :: Delete => Method :: DELETE , HttpMethod :: Patch => Method :: PATCH , } ; let mut request = self . client . request (method , & self . endpoint) ; for (k , v) in & self . headers { request = request . header (k , v) ; } if let Some (auth) = & self . auth { request = match auth { AuthConfig :: Bearer { token } => request . bearer_auth (token) , AuthConfig :: Basic { username , password } => { request . basic_auth (username , Some (password)) } AuthConfig :: ApiKey { key , header } => request . header (header , key) , } ; } if ! input . query . is_empty () { request = request . query (& input . query) ; } if let Some (body) = input . body { request = request . json (& body) ; } let response = request . send () . await . map_err (| e | Error :: Http (format ! ("Request failed: {}" , e))) ? ; let status = response . status () . as_u16 () ; let mut headers = HashMap :: new () ; for (k , v) in response . headers () { if let Ok (v_str) = v . to_str () { headers . insert (k . to_string () , v_str . to_string ()) ; } } let body = response . json :: < serde_json :: Value > () . await . unwrap_or (serde_json :: json ! ({ })) ; Ok (HttpOutput { status , body , headers , }) } } . self_ty`

### ./crates/pforge-runtime/src/handlers/mod.rs

**File Complexity**: 1 | **Functions**: 0


### ./crates/pforge-runtime/src/handlers/pipeline.rs

**File Complexity**: 1 | **Functions**: 0

- **Struct**: `PipelineHandler` [fields: 1]
- **Struct**: `PipelineStep` [fields: 5]
- **Enum**: `ErrorPolicy` [variants: 2]
- **Struct**: `PipelineInput` [fields: 1]
- **Struct**: `PipelineOutput` [fields: 2]
- **Struct**: `StepResult` [fields: 4]
- **Impl**: `impl PipelineHandler { pub fn new (steps : Vec < PipelineStep >) -> Self { Self { steps } } pub async fn execute (& self , input : PipelineInput , registry : & HandlerRegistry ,) -> Result < PipelineOutput > { let mut variables = input . variables ; let mut results = Vec :: new () ; for step in & self . steps { if let Some (condition) = & step . condition { if ! self . evaluate_condition (condition , & variables) { continue ; } } let step_input = if let Some (input_template) = & step . input { self . interpolate_variables (input_template , & variables) } else { serde_json :: json ! ({ }) } ; let step_result = match registry . dispatch (& step . tool , & serde_json :: to_vec (& step_input) ?) . await { Ok (output) => { let output_value : serde_json :: Value = serde_json :: from_slice (& output) ? ; if let Some (var_name) = & step . output_var { variables . insert (var_name . clone () , output_value . clone ()) ; } StepResult { tool : step . tool . clone () , success : true , output : Some (output_value) , error : None , } } Err (e) => { let result = StepResult { tool : step . tool . clone () , success : false , output : None , error : Some (e . to_string ()) , } ; if step . error_policy == ErrorPolicy :: FailFast { results . push (result) ; return Err (e) ; } result } } ; results . push (step_result) ; } Ok (PipelineOutput { results , variables }) } fn evaluate_condition (& self , condition : & str , variables : & HashMap < String , serde_json :: Value >) -> bool { if let Some (var_name) = condition . strip_prefix ('!') { ! variables . contains_key (var_name) } else { variables . contains_key (condition) } } fn interpolate_variables (& self , template : & serde_json :: Value , variables : & HashMap < String , serde_json :: Value > ,) -> serde_json :: Value { match template { serde_json :: Value :: String (s) => { let mut result = s . clone () ; for (key , value) in variables { let pattern = format ! ("{{{{{}}}}}" , key) ; if let Some (value_str) = value . as_str () { result = result . replace (& pattern , value_str) ; } } serde_json :: Value :: String (result) } serde_json :: Value :: Object (obj) => { let mut new_obj = serde_json :: Map :: new () ; for (k , v) in obj { new_obj . insert (k . clone () , self . interpolate_variables (v , variables)) ; } serde_json :: Value :: Object (new_obj) } serde_json :: Value :: Array (arr) => { let new_arr : Vec < _ > = arr . iter () . map (| v | self . interpolate_variables (v , variables)) . collect () ; serde_json :: Value :: Array (new_arr) } other => other . clone () , } } } . self_ty`

### ./crates/pforge-runtime/src/handlers/wrappers.rs

**File Complexity**: 1 | **Functions**: 0

- **Impl**: `Handler` for `# [async_trait] impl Handler for CliHandler { type Input = CliInput ; type Output = CliOutput ; type Error = Error ; async fn handle (& self , input : Self :: Input) -> Result < Self :: Output > { self . execute (input) . await } } . self_ty`
- **Impl**: `Handler` for `# [async_trait] impl Handler for HttpHandler { type Input = HttpInput ; type Output = HttpOutput ; type Error = Error ; async fn handle (& self , input : Self :: Input) -> Result < Self :: Output > { self . execute (input) . await } } . self_ty`

### ./crates/pforge-runtime/src/lib.rs

**File Complexity**: 1 | **Functions**: 0


### ./crates/pforge-runtime/src/middleware.rs

**File Complexity**: 1 | **Functions**: 0

- **Trait**: `Middleware`
- **Struct**: `MiddlewareChain` [fields: 1]
- **Impl**: `impl MiddlewareChain { pub fn new () -> Self { Self { middlewares : Vec :: new () , } } # [doc = " Add middleware to the chain"] pub fn add (& mut self , middleware : Arc < dyn Middleware >) { self . middlewares . push (middleware) ; } # [doc = " Execute middleware chain around a handler"] pub async fn execute < F , Fut > (& self , mut request : Value , handler : F ,) -> Result < Value > where F : FnOnce (Value) -> Fut , Fut : std :: future :: Future < Output = Result < Value > > , { for middleware in & self . middlewares { request = middleware . before (request) . await ? ; } let result = handler (request . clone ()) . await ; match result { Ok (mut response) => { for middleware in self . middlewares . iter () . rev () { response = middleware . after (request . clone () , response) . await ? ; } Ok (response) } Err (error) => { let mut current_error = error ; for middleware in self . middlewares . iter () . rev () { match middleware . on_error (request . clone () , current_error) . await { Ok (recovery_response) => return Ok (recovery_response) , Err (new_error) => current_error = new_error , } } Err (current_error) } } } } . self_ty`
- **Impl**: `Default` for `impl Default for MiddlewareChain { fn default () -> Self { Self :: new () } } . self_ty`
- **Struct**: `LoggingMiddleware` [fields: 1]
- **Impl**: `impl LoggingMiddleware { pub fn new (tag : impl Into < String >) -> Self { Self { tag : tag . into () } } } . self_ty`
- **Impl**: `Middleware` for `# [async_trait :: async_trait] impl Middleware for LoggingMiddleware { async fn before (& self , request : Value) -> Result < Value > { eprintln ! ("[{}] Request: {}" , self . tag , serde_json :: to_string (& request) . unwrap_or_default ()) ; Ok (request) } async fn after (& self , _request : Value , response : Value) -> Result < Value > { eprintln ! ("[{}] Response: {}" , self . tag , serde_json :: to_string (& response) . unwrap_or_default ()) ; Ok (response) } async fn on_error (& self , _request : Value , error : Error) -> Result < Value > { eprintln ! ("[{}] Error: {}" , self . tag , error) ; Err (error) } } . self_ty`
- **Struct**: `ValidationMiddleware` [fields: 1]
- **Impl**: `impl ValidationMiddleware { pub fn new (required_fields : Vec < String >) -> Self { Self { required_fields } } } . self_ty`
- **Impl**: `Middleware` for `# [async_trait :: async_trait] impl Middleware for ValidationMiddleware { async fn before (& self , request : Value) -> Result < Value > { if let Value :: Object (obj) = & request { for field in & self . required_fields { if ! obj . contains_key (field) { return Err (Error :: Handler (format ! ("Missing required field: {}" , field))) ; } } } Ok (request) } } . self_ty`
- **Struct**: `TransformMiddleware` [fields: 2]
- **Impl**: `impl < BeforeFn , AfterFn > TransformMiddleware < BeforeFn , AfterFn > where BeforeFn : Fn (Value) -> Result < Value > + Send + Sync , AfterFn : Fn (Value) -> Result < Value > + Send + Sync , { pub fn new (before_fn : BeforeFn , after_fn : AfterFn) -> Self { Self { before_fn , after_fn } } } . self_ty`
- **Impl**: `Middleware` for `# [async_trait :: async_trait] impl < BeforeFn , AfterFn > Middleware for TransformMiddleware < BeforeFn , AfterFn > where BeforeFn : Fn (Value) -> Result < Value > + Send + Sync , AfterFn : Fn (Value) -> Result < Value > + Send + Sync , { async fn before (& self , request : Value) -> Result < Value > { (self . before_fn) (request) } async fn after (& self , _request : Value , response : Value) -> Result < Value > { (self . after_fn) (response) } } . self_ty`
- **Struct**: `tests::TestMiddleware` [fields: 1]
- **Impl**: `Middleware` for `# [async_trait :: async_trait] impl Middleware for TestMiddleware { async fn before (& self , mut request : Value) -> Result < Value > { if let Value :: Object (ref mut obj) = request { obj . insert (format ! ("{}_before" , self . tag) , Value :: Bool (true) ,) ; } Ok (request) } async fn after (& self , _request : Value , mut response : Value) -> Result < Value > { if let Value :: Object (ref mut obj) = response { obj . insert (format ! ("{}_after" , self . tag) , Value :: Bool (true) ,) ; } Ok (response) } } . self_ty`
- **Function**: `tests::test_middleware_chain_execution_order` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_validation_middleware` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_transform_middleware` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_error_handling_middleware` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Struct**: `tests::RecoveryMiddleware` [fields: 0]
- **Impl**: `Middleware` for `# [async_trait :: async_trait] impl Middleware for RecoveryMiddleware { async fn on_error (& self , _request : Value , error : Error) -> Result < Value > { if error . to_string () . contains ("recoverable") { Ok (json ! ({ "recovered" : true })) } else { Err (error) } } } . self_ty`
- **Function**: `tests::test_multiple_middleware_composition` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-runtime/src/prompt.rs

**File Complexity**: 1 | **Functions**: 0

- **Struct**: `PromptManager` [fields: 1]
- **Struct**: `PromptEntry` [fields: 3]
- **Impl**: `impl PromptManager { pub fn new () -> Self { Self { prompts : HashMap :: new () , } } # [doc = " Register a prompt definition"] pub fn register (& mut self , def : PromptDef) -> Result < () > { if self . prompts . contains_key (& def . name) { return Err (Error :: Handler (format ! ("Prompt '{}' already registered" , def . name))) ; } self . prompts . insert (def . name . clone () , PromptEntry { description : def . description , template : def . template , arguments : def . arguments , } ,) ; Ok (()) } # [doc = " Render a prompt with given arguments"] pub fn render (& self , name : & str , args : HashMap < String , Value >) -> Result < String > { let entry = self . prompts . get (name) . ok_or_else (| | Error :: Handler (format ! ("Prompt '{}' not found" , name))) ? ; self . validate_arguments (entry , & args) ? ; self . interpolate (& entry . template , & args) } # [doc = " Get prompt metadata"] pub fn get_prompt (& self , name : & str) -> Option < PromptMetadata > { self . prompts . get (name) . map (| entry | PromptMetadata { description : entry . description . clone () , arguments : entry . arguments . clone () , }) } # [doc = " List all registered prompts"] pub fn list_prompts (& self) -> Vec < String > { self . prompts . keys () . cloned () . collect () } # [doc = " Validate arguments against schema"] fn validate_arguments (& self , entry : & PromptEntry , args : & HashMap < String , Value > ,) -> Result < () > { for (arg_name , param_type) in & entry . arguments { let is_required = match param_type { ParamType :: Complex { required , .. } => * required , _ => false , } ; if is_required && ! args . contains_key (arg_name) { return Err (Error :: Handler (format ! ("Required argument '{}' not provided" , arg_name))) ; } } Ok (()) } # [doc = " Interpolate template with argument values"] # [doc = " Supports {{variable}} syntax"] fn interpolate (& self , template : & str , args : & HashMap < String , Value >) -> Result < String > { let mut result = template . to_string () ; for (key , value) in args { let placeholder = format ! ("{{{{{}}}}}" , key) ; let replacement = match value { Value :: String (s) => s . clone () , Value :: Number (n) => n . to_string () , Value :: Bool (b) => b . to_string () , Value :: Null => String :: new () , _ => serde_json :: to_string (value) . map_err (| e | Error :: Handler (format ! ("Failed to serialize value: {}" , e))) ? , } ; result = result . replace (& placeholder , & replacement) ; } if result . contains ("{{") && result . contains ("}}") { let unresolved : Vec < & str > = result . split ("{{") . skip (1) . filter_map (| s | s . split ("}}") . next ()) . collect () ; if ! unresolved . is_empty () { return Err (Error :: Handler (format ! ("Unresolved template variables: {}" , unresolved . join (", ")))) ; } } Ok (result) } } . self_ty`
- **Impl**: `Default` for `impl Default for PromptManager { fn default () -> Self { Self :: new () } } . self_ty`
- **Struct**: `PromptMetadata` [fields: 2]
- **Function**: `tests::test_prompt_registration` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_duplicate_prompt_registration` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_simple_interpolation` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_required_argument_validation` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_unresolved_placeholder` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_get_prompt_metadata` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_complex_value_interpolation` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-runtime/src/recovery.rs

**File Complexity**: 1 | **Functions**: 0

- **Enum**: `CircuitState` [variants: 3]
- **Struct**: `CircuitBreakerConfig` [fields: 3]
- **Impl**: `Default` for `impl Default for CircuitBreakerConfig { fn default () -> Self { Self { failure_threshold : 5 , timeout : Duration :: from_secs (60) , success_threshold : 2 , } } } . self_ty`
- **Struct**: `CircuitBreaker` [fields: 5]
- **Impl**: `impl CircuitBreaker { pub fn new (config : CircuitBreakerConfig) -> Self { Self { config , state : Arc :: new (RwLock :: new (CircuitState :: Closed)) , failure_count : Arc :: new (AtomicUsize :: new (0)) , success_count : Arc :: new (AtomicUsize :: new (0)) , last_failure_time : Arc :: new (RwLock :: new (None)) , } } pub async fn get_state (& self) -> CircuitState { * self . state . read () . await } pub async fn call < F , Fut , T > (& self , operation : F) -> Result < T > where F : FnOnce () -> Fut , Fut : std :: future :: Future < Output = Result < T > > , { let current_state = self . get_state () . await ; match current_state { CircuitState :: Open => { if let Some (last_failure) = * self . last_failure_time . read () . await { if last_failure . elapsed () >= self . config . timeout { * self . state . write () . await = CircuitState :: HalfOpen ; self . success_count . store (0 , Ordering :: SeqCst) ; } else { return Err (Error :: Handler ("Circuit breaker is OPEN" . to_string ())) ; } } } _ => { } } match operation () . await { Ok (result) => { self . on_success () . await ; Ok (result) } Err (error) => { self . on_failure () . await ; Err (error) } } } async fn on_success (& self) { let state = self . get_state () . await ; match state { CircuitState :: HalfOpen => { let successes = self . success_count . fetch_add (1 , Ordering :: SeqCst) + 1 ; if successes >= self . config . success_threshold { * self . state . write () . await = CircuitState :: Closed ; self . failure_count . store (0 , Ordering :: SeqCst) ; self . success_count . store (0 , Ordering :: SeqCst) ; } } CircuitState :: Closed => { self . failure_count . store (0 , Ordering :: SeqCst) ; } _ => { } } } async fn on_failure (& self) { let state = self . get_state () . await ; match state { CircuitState :: Closed => { let failures = self . failure_count . fetch_add (1 , Ordering :: SeqCst) + 1 ; if failures >= self . config . failure_threshold { * self . state . write () . await = CircuitState :: Open ; * self . last_failure_time . write () . await = Some (Instant :: now ()) ; } } CircuitState :: HalfOpen => { * self . state . write () . await = CircuitState :: Open ; * self . last_failure_time . write () . await = Some (Instant :: now ()) ; self . failure_count . store (self . config . failure_threshold , Ordering :: SeqCst) ; } _ => { } } } pub fn get_stats (& self) -> CircuitBreakerStats { CircuitBreakerStats { failure_count : self . failure_count . load (Ordering :: SeqCst) , success_count : self . success_count . load (Ordering :: SeqCst) , } } } . self_ty`
- **Struct**: `CircuitBreakerStats` [fields: 2]
- **Struct**: `FallbackHandler` [fields: 2]
- **Impl**: `impl < F , Fut > FallbackHandler < F , Fut > where F : Fn (Error) -> Fut + Send + Sync , Fut : std :: future :: Future < Output = Result < Value > > + Send , { pub fn new (fallback_fn : F) -> Self { Self { fallback_fn , _phantom : std :: marker :: PhantomData , } } pub async fn handle_error (& self , error : Error) -> Result < Value > { (self . fallback_fn) (error) . await } } . self_ty`
- **Struct**: `ErrorTracker` [fields: 2]
- **Impl**: `impl ErrorTracker { pub fn new () -> Self { Self { total_errors : Arc :: new (AtomicU64 :: new (0)) , errors_by_type : Arc :: new (RwLock :: new (std :: collections :: HashMap :: new ())) , } } pub async fn track_error (& self , error : & Error) { self . total_errors . fetch_add (1 , Ordering :: SeqCst) ; let error_type = self . classify_error (error) ; let mut errors = self . errors_by_type . write () . await ; * errors . entry (error_type) . or_insert (0) += 1 ; } fn classify_error (& self , error : & Error) -> String { match error { Error :: Handler (msg) => { if msg . contains ("timeout") || msg . contains ("timed out") { "timeout" . to_string () } else if msg . contains ("connection") { "connection" . to_string () } else { "handler_error" . to_string () } } _ => "unknown" . to_string () , } } pub fn total_errors (& self) -> u64 { self . total_errors . load (Ordering :: SeqCst) } pub async fn errors_by_type (& self) -> std :: collections :: HashMap < String , u64 > { self . errors_by_type . read () . await . clone () } pub async fn reset (& self) { self . total_errors . store (0 , Ordering :: SeqCst) ; self . errors_by_type . write () . await . clear () ; } } . self_ty`
- **Impl**: `Default` for `impl Default for ErrorTracker { fn default () -> Self { Self :: new () } } . self_ty`
- **Struct**: `RecoveryMiddleware` [fields: 2]
- **Impl**: `impl RecoveryMiddleware { pub fn new () -> Self { Self { circuit_breaker : None , error_tracker : Arc :: new (ErrorTracker :: new ()) , } } pub fn with_circuit_breaker (mut self , config : CircuitBreakerConfig) -> Self { self . circuit_breaker = Some (Arc :: new (CircuitBreaker :: new (config))) ; self } pub fn error_tracker (& self) -> Arc < ErrorTracker > { self . error_tracker . clone () } } . self_ty`
- **Impl**: `Default` for `impl Default for RecoveryMiddleware { fn default () -> Self { Self :: new () } } . self_ty`
- **Impl**: `Middleware` for `# [async_trait :: async_trait] impl Middleware for RecoveryMiddleware { async fn before (& self , request : Value) -> Result < Value > { if let Some (cb) = & self . circuit_breaker { let state = cb . get_state () . await ; if state == CircuitState :: Open { return Err (Error :: Handler ("Circuit breaker is OPEN - service unavailable" . to_string ())) ; } } Ok (request) } async fn on_error (& self , _request : Value , error : Error) -> Result < Value > { self . error_tracker . track_error (& error) . await ; if let Some (cb) = & self . circuit_breaker { cb . on_failure () . await ; } Err (error) } async fn after (& self , _request : Value , response : Value) -> Result < Value > { if let Some (cb) = & self . circuit_breaker { cb . on_success () . await ; } Ok (response) } } . self_ty`
- **Function**: `tests::test_circuit_breaker_closed_to_open` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_circuit_breaker_half_open_recovery` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_circuit_breaker_rejects_when_open` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_error_tracker` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_fallback_handler` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_recovery_middleware_integration` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-runtime/src/registry.rs

**File Complexity**: 1 | **Functions**: 0

- **Struct**: `HandlerRegistry` [fields: 1]
- **Trait**: `HandlerEntry`
- **Struct**: `HandlerEntryImpl` [fields: 1]
- **Impl**: `impl < H : Handler > HandlerEntryImpl < H > { fn new (handler : H) -> Self { Self { handler : Arc :: new (handler) , } } } . self_ty`
- **Impl**: `HandlerEntry` for `impl < H > HandlerEntry for HandlerEntryImpl < H > where H : Handler , H :: Input : 'static , H :: Output : 'static , { fn dispatch (& self , params : & [u8]) -> BoxFuture < 'static , Result < Vec < u8 > > > { let input : H :: Input = match serde_json :: from_slice (params) { Ok (input) => input , Err (e) => return Box :: pin (async move { Err (e . into ()) }) , } ; let handler = self . handler . clone () ; Box :: pin (async move { let output = handler . handle (input) . await . map_err (Into :: into) ? ; serde_json :: to_vec (& output) . map_err (Into :: into) }) } fn input_schema (& self) -> schemars :: schema :: RootSchema { H :: input_schema () } fn output_schema (& self) -> schemars :: schema :: RootSchema { H :: output_schema () } } . self_ty`
- **Impl**: `impl HandlerRegistry { # [doc = " Create new empty registry"] pub fn new () -> Self { Self { handlers : FxHashMap :: default () , } } # [doc = " Register a handler with a name"] pub fn register < H > (& mut self , name : impl Into < String > , handler : H) where H : Handler , H :: Input : 'static , H :: Output : 'static , { let entry = HandlerEntryImpl :: new (handler) ; self . handlers . insert (name . into () , Arc :: new (entry)) ; } # [doc = " Check if handler exists"] pub fn has_handler (& self , name : & str) -> bool { self . handlers . contains_key (name) } # [doc = " Dispatch to a handler by name"] # [inline (always)] pub async fn dispatch (& self , tool : & str , params : & [u8]) -> Result < Vec < u8 > > { match self . handlers . get (tool) { Some (handler) => handler . dispatch (params) . await , None => Err (Error :: ToolNotFound (tool . to_string ())) , } } # [doc = " Get number of registered handlers"] pub fn len (& self) -> usize { self . handlers . len () } # [doc = " Check if registry is empty"] pub fn is_empty (& self) -> bool { self . handlers . is_empty () } # [doc = " Get input schema for a tool"] pub fn get_input_schema (& self , tool : & str) -> Option < schemars :: schema :: RootSchema > { self . handlers . get (tool) . map (| h | h . input_schema ()) } # [doc = " Get output schema for a tool"] pub fn get_output_schema (& self , tool : & str) -> Option < schemars :: schema :: RootSchema > { self . handlers . get (tool) . map (| h | h . output_schema ()) } } . self_ty`
- **Impl**: `Default` for `impl Default for HandlerRegistry { fn default () -> Self { Self :: new () } } . self_ty`

### ./crates/pforge-runtime/src/resource.rs

**File Complexity**: 1 | **Functions**: 0

- **Trait**: `ResourceHandler`
- **Struct**: `ResourceManager` [fields: 1]
- **Struct**: `ResourceEntry` [fields: 5]
- **Impl**: `impl ResourceManager { pub fn new () -> Self { Self { resources : Vec :: new () , } } # [doc = " Register a resource with URI template matching"] pub fn register (& mut self , def : ResourceDef , handler : Arc < dyn ResourceHandler > ,) -> Result < () > { let (pattern , param_names) = Self :: compile_uri_template (& def . uri_template) ? ; self . resources . push (ResourceEntry { uri_template : def . uri_template , pattern , param_names , supports : def . supports , handler , }) ; Ok (()) } # [doc = " Match URI and extract parameters (internal use)"] fn match_uri (& self , uri : & str) -> Option < (& ResourceEntry , HashMap < String , String >) > { for entry in & self . resources { if let Some (captures) = entry . pattern . captures (uri) { let mut params = HashMap :: new () ; for (i , name) in entry . param_names . iter () . enumerate () { if let Some (value) = captures . get (i + 1) { params . insert (name . clone () , value . as_str () . to_string ()) ; } } return Some ((entry , params)) ; } } None } # [doc = " Read resource by URI"] pub async fn read (& self , uri : & str) -> Result < Vec < u8 > > { let (entry , params) = self . match_uri (uri) . ok_or_else (| | Error :: Handler (format ! ("No resource matches URI: {}" , uri))) ? ; if ! entry . supports . contains (& ResourceOperation :: Read) { return Err (Error :: Handler (format ! ("Resource {} does not support read operation" , entry . uri_template))) ; } entry . handler . read (uri , params) . await } # [doc = " Write resource by URI"] pub async fn write (& self , uri : & str , content : Vec < u8 >) -> Result < () > { let (entry , params) = self . match_uri (uri) . ok_or_else (| | Error :: Handler (format ! ("No resource matches URI: {}" , uri))) ? ; if ! entry . supports . contains (& ResourceOperation :: Write) { return Err (Error :: Handler (format ! ("Resource {} does not support write operation" , entry . uri_template))) ; } entry . handler . write (uri , params , content) . await } # [doc = " Subscribe to resource changes"] pub async fn subscribe (& self , uri : & str) -> Result < () > { let (entry , params) = self . match_uri (uri) . ok_or_else (| | Error :: Handler (format ! ("No resource matches URI: {}" , uri))) ? ; if ! entry . supports . contains (& ResourceOperation :: Subscribe) { return Err (Error :: Handler (format ! ("Resource {} does not support subscribe operation" , entry . uri_template))) ; } entry . handler . subscribe (uri , params) . await } # [doc = " Compile URI template to regex pattern"] # [doc = " Example: \"file:///{path}\" -> r\"^file:///(.+)$\" with param_names = [\"path\"]"] # [doc = " Uses non-greedy matching to handle multiple parameters correctly"] fn compile_uri_template (template : & str) -> Result < (Regex , Vec < String >) > { let mut pattern = String :: from ("^") ; let mut param_names = Vec :: new () ; let mut chars = template . chars () . peekable () ; while let Some (ch) = chars . next () { if ch == '{' { let mut param_name = String :: new () ; while let Some (& next_ch) = chars . peek () { if next_ch == '}' { chars . next () ; break ; } param_name . push (chars . next () . unwrap ()) ; } if param_name . is_empty () { return Err (Error :: Handler ("Empty parameter name in URI template" . to_string ())) ; } param_names . push (param_name) ; if chars . peek () == Some (& '/') { pattern . push_str ("([^/]+)") ; } else { pattern . push_str ("(.+)") ; } } else { if ".*+?^$[](){}|\\" . contains (ch) { pattern . push ('\\') ; } pattern . push (ch) ; } } pattern . push ('$') ; let regex = Regex :: new (& pattern) . map_err (| e | Error :: Handler (format ! ("Invalid URI template regex: {}" , e))) ? ; Ok ((regex , param_names)) } # [doc = " List all registered resource templates"] pub fn list_templates (& self) -> Vec < & str > { self . resources . iter () . map (| e | e . uri_template . as_str ()) . collect () } } . self_ty`
- **Impl**: `Default` for `impl Default for ResourceManager { fn default () -> Self { Self :: new () } } . self_ty`
- **Struct**: `tests::TestResourceHandler` [fields: 1]
- **Impl**: `ResourceHandler` for `# [async_trait :: async_trait] impl ResourceHandler for TestResourceHandler { async fn read (& self , _uri : & str , _params : HashMap < String , String >) -> Result < Vec < u8 > > { Ok (self . read_response . clone ()) } async fn write (& self , _uri : & str , _params : HashMap < String , String > , _content : Vec < u8 >) -> Result < () > { Ok (()) } } . self_ty`
- **Function**: `tests::test_uri_template_compilation` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_uri_template_multiple_params` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_resource_registration_and_matching` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_resource_read` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_resource_unsupported_operation` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-runtime/src/server.rs

**File Complexity**: 1 | **Functions**: 0

- **Struct**: `McpServer` [fields: 2]
- **Impl**: `impl McpServer { # [doc = " Create a new MCP server from configuration"] pub fn new (config : ForgeConfig) -> Self { Self { config , registry : Arc :: new (RwLock :: new (HandlerRegistry :: new ())) , } } # [doc = " Register all handlers from configuration"] pub async fn register_handlers (& self) -> Result < () > { let mut registry = self . registry . write () . await ; for tool in & self . config . tools { match tool { pforge_config :: ToolDef :: Native { name , .. } => { eprintln ! ("Note: Native handler '{}' requires handler implementation" , name) ; } pforge_config :: ToolDef :: Cli { name , command , args , cwd , env , stream , .. } => { use crate :: handlers :: cli :: CliHandler ; let handler = CliHandler :: new (command . clone () , args . clone () , cwd . clone () , env . clone () , None , * stream ,) ; registry . register (name , handler) ; eprintln ! ("Registered CLI handler: {}" , name) ; } pforge_config :: ToolDef :: Http { name , endpoint , method , headers , auth , .. } => { use crate :: handlers :: http :: { AuthConfig as HttpAuthConfig , HttpHandler , HttpMethod as HandlerHttpMethod } ; let handler_method = match method { pforge_config :: HttpMethod :: Get => HandlerHttpMethod :: Get , pforge_config :: HttpMethod :: Post => HandlerHttpMethod :: Post , pforge_config :: HttpMethod :: Put => HandlerHttpMethod :: Put , pforge_config :: HttpMethod :: Delete => HandlerHttpMethod :: Delete , pforge_config :: HttpMethod :: Patch => HandlerHttpMethod :: Patch , } ; let handler_auth = auth . as_ref () . map (| a | match a { pforge_config :: AuthConfig :: Bearer { token } => { HttpAuthConfig :: Bearer { token : token . clone () , } } pforge_config :: AuthConfig :: Basic { username , password } => { HttpAuthConfig :: Basic { username : username . clone () , password : password . clone () , } } pforge_config :: AuthConfig :: ApiKey { key , header } => { HttpAuthConfig :: ApiKey { key : key . clone () , header : header . clone () , } } }) ; let handler = HttpHandler :: new (endpoint . clone () , handler_method , headers . clone () , handler_auth ,) ; registry . register (name , handler) ; eprintln ! ("Registered HTTP handler: {}" , name) ; } pforge_config :: ToolDef :: Pipeline { name , .. } => { eprintln ! ("Note: Pipeline handler '{}' pending implementation" , name) ; } } } Ok (()) } # [doc = " Run the MCP server"] pub async fn run (& self) -> Result < () > { eprintln ! ("Starting MCP server: {} v{}" , self . config . forge . name , self . config . forge . version) ; eprintln ! ("Transport: {:?}" , self . config . forge . transport) ; eprintln ! ("Tools registered: {}" , self . config . tools . len ()) ; self . register_handlers () . await ? ; eprintln ! ("\n⚠ MCP protocol loop not yet implemented") ; eprintln ! ("Server configuration loaded and handlers registered successfully") ; eprintln ! ("Press Ctrl+C to exit") ; tokio :: signal :: ctrl_c () . await . map_err (| e | Error :: Io (e)) ? ; eprintln ! ("\nShutting down...") ; Ok (()) } # [doc = " Get the handler registry (for testing)"] pub fn registry (& self) -> Arc < RwLock < HandlerRegistry > > { self . registry . clone () } } . self_ty`

### ./crates/pforge-runtime/src/state.rs

**File Complexity**: 1 | **Functions**: 0

- **Trait**: `StateManager`
- **Struct**: `SledStateManager` [fields: 1]
- **Impl**: `impl SledStateManager { pub fn new (path : & str) -> Result < Self > { let db = sled :: open (path) . map_err (| e | Error :: Handler (format ! ("Sled open failed: {}" , e))) ? ; Ok (Self { db }) } } . self_ty`
- **Impl**: `StateManager` for `# [async_trait] impl StateManager for SledStateManager { async fn get (& self , key : & str) -> Result < Option < Vec < u8 > > > { let value = self . db . get (key) . map_err (| e | Error :: Handler (format ! ("Sled get failed: {}" , e))) ? ; Ok (value . map (| v | v . to_vec ())) } async fn set (& self , key : & str , value : Vec < u8 > , _ttl : Option < Duration >) -> Result < () > { self . db . insert (key , value) . map_err (| e | Error :: Handler (format ! ("Sled insert failed: {}" , e))) ? ; self . db . flush () . map_err (| e | Error :: Handler (format ! ("Sled flush failed: {}" , e))) ? ; Ok (()) } async fn delete (& self , key : & str) -> Result < () > { self . db . remove (key) . map_err (| e | Error :: Handler (format ! ("Sled remove failed: {}" , e))) ? ; Ok (()) } async fn exists (& self , key : & str) -> Result < bool > { let exists = self . db . contains_key (key) . map_err (| e | Error :: Handler (format ! ("Sled contains_key failed: {}" , e))) ? ; Ok (exists) } } . self_ty`
- **Struct**: `MemoryStateManager` [fields: 1]
- **Impl**: `impl MemoryStateManager { pub fn new () -> Self { Self { store : dashmap :: DashMap :: new () , } } } . self_ty`
- **Impl**: `Default` for `impl Default for MemoryStateManager { fn default () -> Self { Self :: new () } } . self_ty`
- **Impl**: `StateManager` for `# [async_trait] impl StateManager for MemoryStateManager { async fn get (& self , key : & str) -> Result < Option < Vec < u8 > > > { Ok (self . store . get (key) . map (| v | v . clone ())) } async fn set (& self , key : & str , value : Vec < u8 > , _ttl : Option < Duration >) -> Result < () > { self . store . insert (key . to_string () , value) ; Ok (()) } async fn delete (& self , key : & str) -> Result < () > { self . store . remove (key) ; Ok (()) } async fn exists (& self , key : & str) -> Result < bool > { Ok (self . store . contains_key (key)) } } . self_ty`
- **Function**: `tests::test_memory_state_basic` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 2 items] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_sled_state_basic` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 2 items] [churn: low(1)] [tdg: 2.5]

### ./crates/pforge-runtime/src/timeout.rs

**File Complexity**: 3 | **Functions**: 2

- **Struct**: `TimeoutMiddleware` [fields: 1]
- **Impl**: `impl TimeoutMiddleware { pub fn new (duration : Duration) -> Self { Self { duration } } pub fn from_millis (millis : u64) -> Self { Self :: new (Duration :: from_millis (millis)) } pub fn from_secs (secs : u64) -> Self { Self :: new (Duration :: from_secs (secs)) } pub fn duration (& self) -> Duration { self . duration } } . self_ty`
- **Impl**: `Middleware` for `# [async_trait :: async_trait] impl Middleware for TimeoutMiddleware { async fn before (& self , request : Value) -> Result < Value > { Ok (request) } async fn after (& self , _request : Value , response : Value) -> Result < Value > { Ok (response) } } . self_ty`
- **Struct**: `RetryPolicy` [fields: 5]
- **Impl**: `impl RetryPolicy { pub fn new (max_attempts : u32) -> Self { Self { max_attempts , initial_backoff : Duration :: from_millis (100) , max_backoff : Duration :: from_secs (30) , backoff_multiplier : 2.0 , use_jitter : true , } } pub fn with_backoff (mut self , initial : Duration , max : Duration) -> Self { self . initial_backoff = initial ; self . max_backoff = max ; self } pub fn with_multiplier (mut self , multiplier : f64) -> Self { self . backoff_multiplier = multiplier ; self } pub fn with_jitter (mut self , use_jitter : bool) -> Self { self . use_jitter = use_jitter ; self } # [doc = " Calculate backoff duration for given attempt"] pub fn backoff_duration (& self , attempt : u32) -> Duration { let base_duration = self . initial_backoff . as_millis () as f64 * self . backoff_multiplier . powi (attempt as i32) ; let capped = base_duration . min (self . max_backoff . as_millis () as f64) ; let duration = if self . use_jitter { let jitter = rand :: random :: < f64 > () * capped * 0.1 ; Duration :: from_millis ((capped + jitter) as u64) } else { Duration :: from_millis (capped as u64) } ; duration } # [doc = " Check if error is retryable"] pub fn is_retryable (& self , error : & Error) -> bool { match error { Error :: Handler (msg) => { msg . contains ("timeout") || msg . contains ("timed out") || msg . contains ("connection") || msg . contains ("temporary") } _ => false , } } } . self_ty`
- **Impl**: `Default` for `impl Default for RetryPolicy { fn default () -> Self { Self :: new (3) } } . self_ty`
- **Struct**: `RetryMiddleware` [fields: 1]
- **Impl**: `impl RetryMiddleware { pub fn new (policy : RetryPolicy) -> Self { Self { policy } } pub fn with_max_attempts (max_attempts : u32) -> Self { Self :: new (RetryPolicy :: new (max_attempts)) } pub fn policy (& self) -> & RetryPolicy { & self . policy } } . self_ty`
- **Impl**: `Middleware` for `# [async_trait :: async_trait] impl Middleware for RetryMiddleware { async fn on_error (& self , _request : Value , error : Error) -> Result < Value > { Err (error) } } . self_ty`
- **Function**: `retry_with_policy` [complexity: 5] [cognitive: 16] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `with_timeout` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_retry_policy_backoff` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_retry_policy_max_backoff` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_retry_with_policy_success` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_retry_with_policy_max_attempts` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_retry_non_retryable_error` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_with_timeout_success` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_with_timeout_exceeded` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_combined_timeout_and_retry` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./examples/hello-world/pforge.yaml


### ./examples/hello-world/src/handlers/greet.rs

**File Complexity**: 1 | **Functions**: 1

- **Struct**: `GreetInput` [fields: 2]
- **Function**: `default_greeting` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Struct**: `GreetOutput` [fields: 1]
- **Struct**: `GreetHandler` [fields: 0]
- **Impl**: `Handler` for `# [async_trait :: async_trait] impl Handler for GreetHandler { type Input = GreetInput ; type Output = GreetOutput ; type Error = pforge_runtime :: Error ; async fn handle (& self , input : Self :: Input) -> Result < Self :: Output > { Ok (GreetOutput { message : format ! ("{}, {}!" , input . greeting , input . name) , }) } } . self_ty`
- **Function**: `tests::test_greet_default` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `tests::test_greet_custom` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./examples/hello-world/src/handlers/mod.rs

**File Complexity**: 1 | **Functions**: 0


### ./examples/hello-world/src/main.rs

**File Complexity**: 2 | **Functions**: 1

- **Function**: `main` [complexity: 2] [cognitive: 1] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./examples/rest-api-proxy/pforge.yaml


### ./examples/rest-api-proxy/src/main.rs

**File Complexity**: 1 | **Functions**: 1

- **Function**: `main` [complexity: 1] [cognitive: 0] [big-o: O(1)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./roadmap.yaml


### ./scripts/batch_create_tickets.sh

- **Function**: `batch_create_tickets::for` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::case` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::2003)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::2004)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::2005)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::2006)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::2007)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::2008)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::2009)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::2010)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::esac` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::cat` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Phase**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Priority**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Time**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Status**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::Implement` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::See` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::TDD` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::Working` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::EOF` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::done` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::for` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::case` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3001)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3002)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3003)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3004)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3005)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3006)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3007)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3008)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3009)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::3010)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::esac` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::cat` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Phase**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Priority**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Time**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Status**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::Implement` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::See` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::Comprehensive` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::Quality` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::EOF` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::done` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::for` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::case` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4001)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4002)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4003)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4004)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4005)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4006)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4007)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4008)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4009)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::4010)` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::esac` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::cat` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Phase**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Priority**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Time**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::**Status**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::$title` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::See` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::Complete,` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::Production` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::EOF` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::done` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::echo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `batch_create_tickets::ls` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./scripts/create_remaining_tickets.sh

- **Function**: `create_remaining_tickets::cat` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::**Phase**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::**Priority**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::**Time**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::**Status**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::Implement` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::EOF` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::cat` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::**Phase**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::**Priority**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::**Time**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::**Status**:` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::Implement` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::-` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::EOF` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `create_remaining_tickets::echo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

### ./scripts/pre-commit.sh

- **Function**: `pre-commit::set` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::echo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::echo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::cargo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::echo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::cargo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::echo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::cargo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::if` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::echo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::pmat` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::pmat` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::pmat` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::fi` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
- **Function**: `pre-commit::echo` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]

## Key Components

Key architectural components identified in the codebase.

## Big-O Complexity Analysis

Complexity analysis results integrated in function annotations above.

## Entropy Analysis

Code entropy and organization metrics.

## Provability Analysis

Formal verification and provability insights.

## Graph Metrics

Dependency graph and PageRank analysis.

## Technical Debt Gradient (TDG)

Technical debt progression and accumulation patterns.

## Dead Code Analysis

Unused code detection and removal recommendations.

## Self-Admitted Technical Debt (SATD)

TODO, FIXME, and HACK comments indicating technical debt.

## Quality Insights

Overall code quality assessment and trends.

## Recommendations

Actionable suggestions for code improvement.

