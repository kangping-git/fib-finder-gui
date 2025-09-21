use bstr::ByteSlice;
use ibig::IBig;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering},
};
use std::thread;

struct Matrix2x2 {
    a11: IBig,
    a12: IBig,
    a21: IBig,
    a22: IBig,
}

impl Matrix2x2 {
    fn new(a11: IBig, a12: IBig, a21: IBig, a22: IBig) -> Self {
        Matrix2x2 { a11, a12, a21, a22 }
    }
}

fn dot(x: &Matrix2x2, y: &Matrix2x2) -> Matrix2x2 {
    Matrix2x2 {
        a11: (&x.a11 * &y.a11) + (&x.a12 * &y.a21),
        a12: (&x.a11 * &y.a12) + (&x.a12 * &y.a22),
        a21: (&x.a21 * &y.a11) + (&x.a22 * &y.a21),
        a22: (&x.a21 * &y.a12) + (&x.a22 * &y.a22),
    }
}

fn calc_fib_x(mut x: i64) -> (IBig, IBig) {
    let mut matrix: Matrix2x2 =
        Matrix2x2::new(IBig::from(1), IBig::from(0), IBig::from(0), IBig::from(1));
    let mut matrix2: Matrix2x2 =
        Matrix2x2::new(IBig::from(1), IBig::from(1), IBig::from(1), IBig::from(0));
    while x > 0 {
        if x % 2 == 1 {
            matrix = dot(&matrix, &matrix2);
        }
        matrix2 = dot(&matrix2, &matrix2);
        x /= 2;
    }
    (matrix.a11, matrix.a12)
}

pub struct CalcStatus {
    pub status: Vec<Arc<Status>>,
    pub is_stop: Arc<AtomicBool>,
    pub ans: Arc<AtomicI64>,
}
pub struct Status {
    pub is_finished: AtomicBool,
    pub place: AtomicI64,
    pub percent: AtomicUsize,
}
fn search_part(
    result: Arc<Status>,
    is_find: Arc<AtomicBool>,
    is_stop: Arc<AtomicBool>,
    x_start: Arc<AtomicI64>,
    found_idx: Arc<AtomicI64>,
    needle: Arc<Vec<u8>>,
    chunk: i64,
) {
    let needle = needle.to_vec();
    while !is_find.load(Ordering::Relaxed) {
        let beg = x_start.fetch_add(chunk, Ordering::Relaxed);
        let end = beg + chunk;

        let (mut x, mut y) = calc_fib_x(beg + 1);

        if y.to_string().as_bytes().contains_str(&needle) {
            is_find.store(true, Ordering::Relaxed);
            let now = found_idx.load(Ordering::Relaxed);
            if now == -1 || now > beg {
                found_idx.store(beg, Ordering::Relaxed);
            }
            result.place.store(beg, Ordering::Relaxed);
            break;
        }
        if x.to_string().as_bytes().contains_str(&needle) {
            is_find.store(true, Ordering::Relaxed);
            let now = found_idx.load(Ordering::Relaxed);
            if now == -1 || now > beg + 1 {
                found_idx.store(beg + 1, Ordering::Relaxed);
            }
            result.place.store(beg + 1, Ordering::Relaxed);
            break;
        }

        for i in beg + 2..end {
            let next = &x + &y;
            y = x;
            x = next;

            if x.to_string().as_bytes().contains_str(&needle) {
                is_find.store(true, Ordering::Relaxed);
                let now = found_idx.load(Ordering::Relaxed);
                if now == -1 || now > i + 1 {
                    found_idx.store(i + 1, Ordering::Relaxed);
                }
                result.place.store(i + 1, Ordering::Relaxed);
                break;
            }

            if i % (chunk / 100) == 0 {
                if is_stop.load(Ordering::Relaxed) {
                    break;
                };
                if is_find.load(Ordering::Relaxed) && end > found_idx.load(Ordering::Relaxed) {
                    break;
                }
                let prog = ((i - beg) / (chunk / 100)) as usize;
                let prog = prog.min(100);
                result.place.store(i, Ordering::Relaxed);
                result.percent.store(prog, Ordering::Relaxed);
            }
        }
    }
    result.is_finished.store(true, Ordering::Relaxed);
}

pub fn calc(needle: String, threads: usize, chunk: i64) -> CalcStatus {
    let found = Arc::new(AtomicBool::new(false));
    let found_idx = Arc::new(AtomicI64::new(-1));
    let counter = Arc::new(AtomicI64::new(0));
    let is_stopped = Arc::new(AtomicBool::new(false));

    let needle = Arc::new(needle.into_bytes());

    let mut thread_pool = Vec::with_capacity(threads);
    let mut results: Vec<Arc<Status>> = Vec::with_capacity(threads);

    for _ in 0..threads {
        let f = Arc::clone(&found);
        let c = Arc::clone(&counter);
        let idx = Arc::clone(&found_idx);
        let n = Arc::clone(&needle);
        let s = Arc::clone(&is_stopped);
        let status_arc = Arc::new(Status {
            is_finished: AtomicBool::new(false),
            place: AtomicI64::new(0),
            percent: AtomicUsize::new(0),
        });
        results.push(Arc::clone(&status_arc));

        thread_pool.push(thread::spawn(move || {
            search_part(status_arc, f, s, c, idx, n, chunk)
        }));
    }
    CalcStatus {
        is_stop: is_stopped,
        status: results,
        ans: found_idx,
    }
}
