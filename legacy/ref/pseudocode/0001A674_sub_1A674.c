/* ea:   0x0001A674 */
/* name: sub_1A674 */
/* size: 416 */
void sub_1A674(int a1, ...)
{
  void *v2; // r0
  int v3; // r0
  _DWORD *v4; // r4
  _DWORD *v5; // r0
  int v6; // r0
  _DWORD *v7; // r2
  _DWORD *v8; // r4
  _DWORD v9[4]; // [sp+Ch] [bp-44h] BYREF
  _DWORD v10[9]; // [sp+1Ch] [bp-34h] BYREF
  void *varg_r1; // [sp+44h] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+44h] [bp-Ch]
  int varg_r2; // [sp+48h] [bp-8h]
  int varg_r3; // [sp+4Ch] [bp-4h]
  void *v15; // [sp+50h] [bp+0h] BYREF
  va_list va1; // [sp+50h] [bp+0h]
  int v17; // [sp+54h] [bp+4h]
  int v18; // [sp+58h] [bp+8h]
  va_list va2; // [sp+5Ch] [bp+Ch] BYREF

  va_start(va2, a1);
  va_start(va1, a1);
  va_start(varg_r1a, a1);
  varg_r1 = va_arg(va1, void *);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _DWORD);
  va_copy(va2, va1);
  v15 = va_arg(va2, void *);
  v17 = va_arg(va2, _DWORD);
  v18 = va_arg(va2, _DWORD);
  sub_132FC(v9, (void **)va1);
  sub_132FC(v10, (void **)varg_r1a);
  sub_44004((LPCWSTR)a1);
  v2 = operator new(0x3FCu);
  if ( v2 != nullptr )
    v3 = sub_43B8C(v2, *(_DWORD *)(a1 + 4));
  else
    v3 = 0;
  *(_DWORD *)(a1 + 1020) = v3;
  sub_43D54(v3, a1);
  v4 = sub_132FC(v10, (void **)va2);
  v5 = sub_132FC(v9, (void **)varg_r1a);
  (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(**(_DWORD **)(a1 + 1020) + 4))(
    *(_DWORD *)(a1 + 1020),
    *v5,
    v5[1],
    v5[2],
    *v4,
    v4[1],
    v4[2]);
  if ( operator new(0x34u) != nullptr )
    v6 = sub_45EC0();
  else
    v6 = 0;
  *(_DWORD *)(a1 + 1136) = v6;
  sub_46294();
  *(_DWORD *)(*(_DWORD *)(a1 + 1136) + 24) = 0;
  *(_DWORD *)(*(_DWORD *)(a1 + 1136) + 4) = 1;
  *(_DWORD *)(*(_DWORD *)(a1 + 1136) + 8) = 0;
  *(_DWORD *)(*(_DWORD *)(a1 + 1136) + 28) = 1;
  v7 = *(_DWORD **)(a1 + 1136);
  if ( *v7 == 0 )
    __debugbreak();
  v7[9] = *(_DWORD *)(*v7 + 4);
  v8 = *(_DWORD **)(a1 + 1136);
  if ( *v8 == 0 )
    __debugbreak();
  v8[10] = *(_DWORD *)(*v8 + 8) / v8[1];
  sub_12F48((void **)varg_r1a);
  sub_12F48((void **)va1);
  sub_12F48((void **)va2);
}

