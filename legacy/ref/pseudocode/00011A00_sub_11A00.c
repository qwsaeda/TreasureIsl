/* ea:   0x00011A00 */
/* name: sub_11A00 */
/* size: 248 */
int sub_11A00(int a1, ...)
{
  void *v2; // r0
  int v3; // r0
  _DWORD *v4; // r4
  _DWORD *v5; // r0
  _BYTE v7[16]; // [sp+Ch] [bp-40h] BYREF
  _BYTE v8[32]; // [sp+1Ch] [bp-30h] BYREF
  int varg_r1; // [sp+40h] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+40h] [bp-Ch]
  int varg_r2; // [sp+44h] [bp-8h]
  int varg_r3; // [sp+48h] [bp-4h]
  int v13; // [sp+4Ch] [bp+0h] BYREF
  va_list va1; // [sp+4Ch] [bp+0h]
  int v15; // [sp+50h] [bp+4h]
  int v16; // [sp+54h] [bp+8h]
  va_list va2; // [sp+58h] [bp+Ch] BYREF

  va_start(va2, a1);
  va_start(va1, a1);
  va_start(varg_r1a, a1);
  varg_r1 = va_arg(va1, _DWORD);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _DWORD);
  va_copy(va2, va1);
  v13 = va_arg(va2, _DWORD);
  v15 = va_arg(va2, _DWORD);
  v16 = va_arg(va2, _DWORD);
  v2 = operator new(0x3FCu);
  if ( v2 != nullptr )
    v3 = sub_43B8C(v2, *(_DWORD *)(a1 + 4));
  else
    v3 = 0;
  *(_DWORD *)(a1 + 1020) = v3;
  sub_43D54();
  v4 = (_DWORD *)sub_132FC(v7, va2);
  v5 = (_DWORD *)sub_132FC(v8, (int *)varg_r1a);
  (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(**(_DWORD **)(a1 + 1020) + 4))(
    *(_DWORD *)(a1 + 1020),
    *v5,
    v5[1],
    v5[2],
    *v4,
    v4[1],
    v4[2]);
  sub_132FC(v8, (int *)va1);
  sub_132FC(v7, (int *)varg_r1a);
  sub_44004((LPCWSTR)a1);
  sub_12F48((int *)varg_r1a);
  sub_12F48((int *)va1);
  return sub_12F48(va2);
}

