// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use std::os::raw::c_void;
use ark_bls12_377::{Fr, G1Affine};
use ark_ec::AffineCurve;
//use ark_ff::PrimeField;
use ark_std::Zero;

use core::slice;

#[allow(unused_imports)]
use blst::*;

pub mod util;


#[cfg_attr(feature = "quiet", allow(improper_ctypes), allow(dead_code))]
extern "C" {
    // allocate an MSM context (which holds on to GPU memory)
    // specify the max number of points and max number of batches
    fn MSMAllocContext(maxPoints: u32, maxBatches: u32) -> *mut c_void;
    
    // free the context, release GPU memory and resources
    fn MSMFreeContext(context: *mut c_void) -> i32;
    
    // prepare a set of MSM points
    fn MSMPreprocessPoints(context: *mut c_void, affine_points_ptr: *const G1Affine, points: u32) -> i32;
    
    // run batches of MSM, using points that were prepared earlier.
    fn MSMRun(context: *mut c_void, projective_results: *mut u64, scalars_ptr: *const Fr, scalars: u32) -> i32;
}

pub fn multi_scalar_mult_init<G: AffineCurve>(
    points: &[G],
) -> *mut c_void {

    let max_points = 1<<26;
    let max_batches = 16;
    let npoints = points.len();
    
    let ret = unsafe {
        MSMAllocContext(max_points, max_batches)
    };

    let err = unsafe {
       MSMPreprocessPoints(
          ret, 
          points as *const _ as *const G1Affine,
          npoints as u32,
       )
    };
    
    if err != 0 {
      panic!("Error {} occurred in C code", err);
    }
    ret
}
    
pub fn multi_scalar_mult(
    context: *mut c_void,
    npoints: usize,
    scalars: &[Fr],
) -> Vec<<G1Affine as AffineCurve>::Projective> {
    let nscalars = scalars.len();
    let batch_size = nscalars / npoints;
        
    let mut ret = vec![<G1Affine as AffineCurve>::Projective::zero(); batch_size];
    
    let err = unsafe {      
      MSMRun(
        context, 
        ret.as_mut_ptr() as *mut u64,
        scalars as *const _ as *const Fr,
        nscalars as u32,
      )
    };
    
    if err != 0 {
      panic!("Error {} occurred in C code", err);
    }

    ret
}

#[no_mangle]
pub extern "C" fn multi_scalar_init_wrapper(points_ptr: *const c_void, len: u64) -> *mut c_void {
    let points: &[G1Affine] = unsafe {
        slice::from_raw_parts(points_ptr as *const G1Affine, len as usize)
    };
    multi_scalar_mult_init(points)
}

#[no_mangle]
pub extern "C" fn multi_scalar_mult_wrapper(p: *mut c_void, ctx: *mut c_void, scalars: *const c_void, len: u64) {
    let len = len as usize;
    let scalars: &[Fr] = unsafe {
        slice::from_raw_parts(scalars as *const Fr, len)
    };

    let v = multi_scalar_mult(ctx, len, scalars);
    unsafe {
        *(p as *mut <G1Affine as AffineCurve>::Projective) = v[0];
    }
}
