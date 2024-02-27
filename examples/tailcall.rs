use tailcall::tailcall;

fn factorial(input: u64) -> u64 {
    #[tailcall]
    fn factorial_inner(accumulator: u64, input: u64) -> u64 {
        if input > 0 {
            factorial_inner(accumulator * input, input - 1)
        } else {
            accumulator
        }
    }

    factorial_inner(1, input)
}

fn count_down(input: u64) -> bool {
    #[tailcall]
    fn cd_inner(dummy: bool, input: u64) -> bool {
        if input > 0 {
            println!("{input}");
            cd_inner(true, input - 1)
        } else {
            true
        }
    }

    cd_inner(true, input)
}

// does not work as there is no accumulator
// fn count_down_no_ret(input: u64) {

//     #[tailcall]
//     fn cd_nr_inner(input: u64) {
//         if input > 0 {
//             println!("{input}");
//             cd_nr_inner(input - 1)
//         }
//     }

//     cd_nr_inner(input)
// }

fn main() {
    // TryFrom
    let x = 5;
    println!("factorial({x}) = {}", factorial(x));

    println!("count_down({x}) = {}", count_down(x));
}
