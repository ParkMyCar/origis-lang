fn fib(n: int) int {
    if n <= 1 {
    	1
    } else {
        fib(n - 1) + fib(n - 2)
    }
}
