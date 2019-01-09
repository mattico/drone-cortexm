//! Drone threading system resources.

use crate::{
  map::{
    reg::{mpu, scb},
    res::{mpu::MpuRes, thr::ThrRes},
  },
  reg::prelude::*,
  thr::ThrTokens,
};

static MPU_RESET_TABLE: [u32; 16] = [
  rbar_reset(0),
  0,
  rbar_reset(1),
  0,
  rbar_reset(2),
  0,
  rbar_reset(3),
  0,
  rbar_reset(4),
  0,
  rbar_reset(5),
  0,
  rbar_reset(6),
  0,
  rbar_reset(7),
  0,
];

/// `Thr` driver.
pub struct Thr {
  mpu: MpuRes,
  thr: ThrRes,
}

/// Creates a new `Thr`.
#[macro_export]
macro_rules! drv_thr {
  ($reg:ident) => {
    $crate::drv::thr::Thr::new($crate::res_mpu!($reg), $crate::res_thr!($reg))
  };
}

impl Thr {
  /// Creates a new `Thr`.
  #[inline(always)]
  pub fn new(mpu: MpuRes, thr: ThrRes) -> Self {
    Self { mpu, thr }
  }

  /// Releases the underlying registers.
  #[inline(always)]
  pub fn free(self) -> (MpuRes, ThrRes) {
    let Self { mpu, thr } = self;
    (mpu, thr)
  }

  /// Initialized the Drone threading system, and returns an instance of `T`.
  #[inline(always)]
  pub fn init<T: ThrTokens>(
    self,
    scb_ccr_init: impl for<'a, 'b> FnOnce(
      &'b mut scb::ccr::Hold<'a, Srt>,
    ) -> &'b mut scb::ccr::Hold<'a, Srt>,
  ) -> T {
    self
      .thr
      .scb_ccr
      .store(|r| scb_ccr_init(r).set_stkalign().set_nonbasethrdena());
    unsafe {
      self.mpu_reset();
      T::take()
    }
  }

  #[allow(clippy::used_underscore_binding)]
  #[inline]
  unsafe fn mpu_reset(&self) {
    if self.mpu.mpu_type.load().dregion() == 0 {
      return;
    }
    self.mpu.mpu_ctrl.reset();
    let mut _table_ptr = &MPU_RESET_TABLE;
    asm!("
      ldmia $0!, {r5-r12}
      stmia $1, {r5-r12}
      ldmia $0!, {r5-r12}
      stmia $1, {r5-r12}
    " : "+&rm"(_table_ptr)
      : "r"(mpu::Rbar::<Srt>::ADDRESS)
      : "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12"
      : "volatile");
  }
}

#[allow(clippy::cast_lossless)]
const fn rbar_reset(region: u8) -> u32 {
  1 << 4 | region as u32 & 0b1111
}
