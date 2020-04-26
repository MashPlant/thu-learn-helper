use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::marker::PhantomPinned;

// this is basically copied from `futures::future::try_join_all`
// the reasons that I am writing it myself are: 1. I don't want to depend on `futures` just for this function
// 2. the returned `Vec` of `futures::future::try_join_all` is not used in mu case
pub fn try_join_all<I, E>(i: I) -> TryJoinAll<I::Item> where I: IntoIterator, I::Item: Future<Output=Result<(), E>> {
  TryJoinAll { elems: i.into_iter().map(ElemState::Pending).collect(), _pin: PhantomPinned }
}

pub struct TryJoinAll<E> {
  elems: Box<[ElemState<E>]>,
  _pin: PhantomPinned,
}

enum ElemState<F> {
  Pending(F),
  Done,
}

enum FinalState<E> {
  Pending,
  AllDone,
  Error(E),
}

impl<F, E> Future for TryJoinAll<F> where F: Future<Output=Result<(), E>> {
  type Output = Result<(), E>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    let mut state = FinalState::AllDone;
    for elem in { unsafe { self.get_unchecked_mut() } }.elems.iter_mut() {
      match elem {
        ElemState::Pending(f) => match unsafe { Pin::new_unchecked(f) }.poll(cx) {
          Poll::Pending => state = FinalState::Pending,
          Poll::Ready(Ok(_)) => *elem = ElemState::Done,
          Poll::Ready(Err(e)) => {
            state = FinalState::Error(e);
            break;
          }
        }
        ElemState::Done => {}
      }
    }
    match state {
      FinalState::Pending => Poll::Pending,
      FinalState::AllDone => Poll::Ready(Ok(())),
      FinalState::Error(e) => Poll::Ready(Err(e)),
    }
  }
}