use std::slice::Iter;

use anymap::{CloneAny, Map};

pub type DMap = Map<dyn CloneAny + Send + Sync>;

/// Describe task's running result
pub struct ExecState {
    /// The execution succeed or not
    success: bool,
    /// Return value of the execution.
    retval: Retval,
}

/// Task's return value
pub struct Retval(Option<DMap>);

/// Task's input value
pub struct Inputval(Vec<Option<DMap>>);

impl ExecState {
    /// Get a new [`ExecState`].
    ///
    /// `success`: task finish without panic?
    ///
    /// `retval`: task's return value
    pub fn new(success: bool, retval: Retval) -> Self {
        Self { success, retval }
    }

    /// Get [`ExecState`]'s return value.
    ///
    /// This method will clone [`DMap`] that are stored in [`ExecState`]'s [`Retval`].
    pub fn get_dmap(&self) -> Option<DMap> {
        self.retval.0.clone()
    }

    /// The task execution succeed or not.
    ///
    /// `true` means no panic occurs.
    pub fn success(&self) -> bool {
        self.success
    }
}

impl Retval {
    #[allow(unused)]
    /// Get a new [`Retval`].
    ///
    /// Since the return value may be transfered between threads,
    /// [`Send`], [`Sync`], [`CloneAny`] is needed.
    ///
    /// # Example
    /// ```rust
    /// let retval = Retval::new(123);
    /// ```
    pub fn new<H: Send + Sync + CloneAny>(val: H) -> Self {
        let mut map = DMap::new();
        assert!(map.insert(val).is_none(), "[Error] map insert fails.");
        Self(Some(map))
    }

    /// Get empty [`Retval`].
    ///
    /// # Example
    /// ```rust
    /// let retval = Retval::empty();
    /// ```
    pub fn empty() -> Self {
        Self(None)
    }
}

impl Inputval {
    /// Get a new [`Inputval`], values stored in vector are ordered
    /// by that of the given [`TaskWrapper`]'s `rely_list`.
    pub fn new(vals: Vec<Option<DMap>>) -> Self {
        Self(vals)
    }

    #[allow(unused)]
    /// This method get needed input value from [`Inputval`],
    /// and, it takes an index as input.
    ///
    /// When input from only one task's return value,
    /// just set index `0`. If from muti-tasks' return values, the index depends on
    /// which return value you want. The order of the return values are the same
    /// as you defined in [`input_from`] function.
    ///
    /// # Example
    /// ```rust
    /// // previous definition of `t3`
    /// t3.input_from(&[&t1, &t2]);
    /// // then you wanna get input
    /// let input_from_t1 = input.get(0);
    /// let input_from_t2 = input.get(1);
    /// ```
    pub fn get<H: Send + Sync + CloneAny>(&mut self, index: usize) -> Option<H> {
        if let Some(Some(dmap)) = self.0.get_mut(index) {
            dmap.remove()
        } else {
            None
        }
    }

    /// Since [`Inputval`] can contain mult-input values, and it's implemented
    /// by [`Vec`] actually, of course it can be turned into a iterater.
    pub fn get_iter(&self) -> Iter<Option<Map<dyn CloneAny + Send + Sync>>> {
        self.0.iter()
    }
}
