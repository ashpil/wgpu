#[macro_use]
extern crate pipeline;

// use for the shader! macro
pub use pipeline::shared;
pub use pipeline::wgpu_compute_header;

pub use pipeline::wgpu_compute_header::{compile, read_uvec, run, ComputeShader};

pub use pipeline::shared::{is_gl_builtin, Bindable};

pub use pipeline::context::{ready_to_run, update_bind_context, BindingContext};

pub use static_assertions::const_assert;

async fn execute_gpu() {
    // qualifiers
    // buffer: is a buffer?
    // in: this parameter must be bound to before the program runs
    //     thus it can be read inside of the program scope
    // out: if this parameter is bound, it must be rebound after each iteration
    //      Only out variables can be mutated
    //      Only out variables can be read as a result of the program
    //      If out has been unassigned then an error is raised when it is read
    // loop: one or more of these loop annotations are required per program. Atm, the values bound is assumed to be of equal length and this gives the number of iterations(gl_GlobalInvocationID.x)
    //      the size of any out buffers that need to be created

    const TRIVIAL: (ComputeShader, BindingContext) = compute_shader! {
        [[buffer loop in out] uint[]] indices;
        [[buffer in] uint[]] indices2;
        //[[buffer out] uint[]] result;
        //[... uint] xindex;
        {{
            void main() {
                // uint xindex = gl_GlobalInvocationID.x;
                uint index = gl_GlobalInvocationID.x;
                indices[index] = indices[index]+indices2[index];
            }
        }}
    };

    const S: ComputeShader = TRIVIAL.0;
    const STARTING_BIND_CONTEXT: BindingContext = TRIVIAL.1;

    let (program, mut bindings, mut out_bindings) = compile(&S).await;
    let (_, _, mut out_bindings2) = compile(&S).await;

    let indices_1: Vec<u32> = vec![1, 2, 3, 4];
    let indices_2: Vec<u32> = vec![2, 2, 2, 2];
    let indices2: Vec<u32> = vec![4, 3, 2, 1];

    const BIND_CONTEXT_1: BindingContext = update_bind_context(&STARTING_BIND_CONTEXT, "indices2");
    Bindable::bind(
        &indices2,
        &program,
        &mut bindings,
        &mut out_bindings,
        "indices2".to_string(),
    );
    {
        const BIND_CONTEXT_2: BindingContext = update_bind_context(&BIND_CONTEXT_1, "indices");
        Bindable::bind(
            &indices_1,
            &program,
            &mut bindings,
            &mut out_bindings,
            "indices".to_string(),
        );

        {
            const _: () = ready_to_run(BIND_CONTEXT_2);
            let result1 = run(&program, &mut bindings, out_bindings);
            println!("{:?}", read_uvec(&program, &result1, "indices").await);
        }

        const BIND_CONTEXT_4: BindingContext = update_bind_context(&BIND_CONTEXT_1, "indices");
        Bindable::bind(
            &indices_2,
            &program,
            &mut bindings,
            &mut out_bindings2,
            "indices".to_string(),
        );
        {
            const _: () = ready_to_run(BIND_CONTEXT_4);
            let result1 = run(&program, &mut bindings, out_bindings2);
            println!("{:?}", read_uvec(&program, &result1, "indices").await);
        }
    }
}

fn main() {
    futures::executor::block_on(execute_gpu());
}
