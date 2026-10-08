/* ea:   0x0004E688 */
/* name: sub_4E688 */
/* size: 112 */
void sub_4E688(_DWORD *a1, ...)
{
  _DWORD *v2; // r4
  _DWORD *v3; // r0
  _DWORD v4[4]; // [sp+Ch] [bp-3Ch] BYREF
  _DWORD v5[7]; // [sp+1Ch] [bp-2Ch] BYREF
  void *varg_r1; // [sp+3Ch] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+3Ch] [bp-Ch]
  int varg_r2; // [sp+40h] [bp-8h]
  int varg_r3; // [sp+44h] [bp-4h]
  va_list va1; // [sp+48h] [bp+0h] BYREF

  va_start(va1, a1);
  va_start(varg_r1a, a1);
  varg_r1 = va_arg(va1, void *);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _DWORD);
  v2 = sub_132FC(v4, (void **)va1);
  v3 = sub_132FC(v5, (void **)varg_r1a);
  sub_44004(a1, *v3, v3[1], v3[2], *v2, v2[1], v2[2]);
  sub_12F48((void **)varg_r1a);
  sub_12F48((void **)va1);
}

