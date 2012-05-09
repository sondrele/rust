import syntax::ast;
import lib::llvm::ValueRef;
import common::*;
import build::*;
import base::*;
import shape::llsize_of;

export trans_uniq, make_free_glue, autoderef, duplicate, alloc_uniq;

fn trans_uniq(bcx: block, contents: @ast::expr,
              node_id: ast::node_id, dest: dest) -> block {
    let _icx = bcx.insn_ctxt("uniq::trans_uniq");
    let uniq_ty = node_id_type(bcx, node_id);
    let contents_ty = content_ty(uniq_ty);
    let {box, body} = malloc_unique(bcx, contents_ty);
    add_clean_free(bcx, box, true);
    let bcx = trans_expr_save_in(bcx, contents, body);
    revoke_clean(bcx, box);
    ret store_in_dest(bcx, box, dest);
}

fn alloc_uniq(bcx: block, uniq_ty: ty::t) -> ValueRef {
    let _icx = bcx.insn_ctxt("uniq::alloc_uniq");
    let contents_ty = content_ty(uniq_ty);
    let llty = type_of::type_of(bcx.ccx(), contents_ty);
    let llsz = llsize_of(bcx.ccx(), llty);
    let llptrty = T_ptr(llty);
    shared_malloc(bcx, llptrty, llsz)
}

fn make_free_glue(bcx: block, vptr: ValueRef, t: ty::t)
    -> block {
    let _icx = bcx.insn_ctxt("uniq::make_free_glue");
    with_cond(bcx, IsNotNull(bcx, vptr)) {|bcx|
        let content_ty = content_ty(t);
        let body_ptr = opaque_box_body(bcx, content_ty, vptr);
        let bcx = drop_ty(bcx, body_ptr, content_ty);
        trans_unique_free(bcx, vptr)
    }
}

fn content_ty(t: ty::t) -> ty::t {
    alt ty::get(t).struct {
      ty::ty_uniq({ty: ct, _}) { ct }
      _ { core::unreachable(); }
    }
}

fn autoderef(bcx: block, v: ValueRef, t: ty::t) -> {v: ValueRef, t: ty::t} {
    let content_ty = content_ty(t);
    let v = opaque_box_body(bcx, content_ty, v);
    ret {v: v, t: content_ty};
}

fn duplicate(bcx: block, v: ValueRef, t: ty::t) -> result {
    let _icx = bcx.insn_ctxt("uniq::duplicate");
    let content_ty = content_ty(t);
    let {box: dst_box, body: dst_body} = malloc_unique(bcx, content_ty);

    let src_box = v;
    let src_body = opaque_box_body(bcx, content_ty, src_box);
    let src_body = load_if_immediate(bcx, src_body, content_ty);
    #debug("ST: %?", val_str(bcx.ccx().tn, src_body));
    #debug("DT: %?", val_str(bcx.ccx().tn, dst_body));
    let bcx = copy_val(bcx, INIT, dst_body, src_body, content_ty);

    let src_tydesc_ptr = GEPi(bcx, src_box,
                              [0u, back::abi::box_field_tydesc]);
    let dst_tydesc_ptr = GEPi(bcx, dst_box,
                              [0u, back::abi::box_field_tydesc]);

    let td = Load(bcx, src_tydesc_ptr);
    Store(bcx, td, dst_tydesc_ptr);

    ret rslt(bcx, dst_box);
}