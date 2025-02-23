.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # sp->kernel stack, sscratch->user stack
    # allocate a TrapContext
    addi sp, sp, -34*8
    # save general purpose register
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later by sscratch
    sd x3, 3*8(sp)
    # skip tp(x4) since almost no use
    # save x5-x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # using t0,t1,t2 to save sstatus, sepc and sscratch
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    csrr t2, sscratch
    # note that sscratch points to user stack now
    sd t2, 2*8(sp)
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp
    call trap_handler

__restore:
    # case 1: start app by __restore
    # case 2: back to U mode after handling trap
    # mv sp, a0
    # note that __restore(cx_addr: usize), a0 stores the cx_addr
    # so now, sp->kernel stack, sscratch->user stack
    # restore sstatus, sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # restore general purpose register
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp
    sret