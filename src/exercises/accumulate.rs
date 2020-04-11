pub fn map<F, T, U>(values: Vec<T>, mut f: F) -> Vec<U>
where
    F: FnMut(T) -> U,
{
    let mut v = Vec::with_capacity(values.len());
    for val in values {
        v.push(f(val));
    }
    v
}

pub fn do_twice<F>(mut func: F)
where
    F: FnMut(),
{
    func();
    func();
}
