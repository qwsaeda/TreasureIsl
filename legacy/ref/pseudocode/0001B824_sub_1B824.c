/* ea:   0x0001B824 */
/* name: sub_1B824 */
/* size: 284 */
void sub_1B824(int a1, ...)
{
  void *v2; // r0
  int v3; // r1
  int v4; // r2
  int v5; // r0
  _DWORD *v6; // r1
  _DWORD *v7; // r4
  _DWORD v8[4]; // [sp+Ch] [bp-3Ch] BYREF
  _DWORD v9[7]; // [sp+1Ch] [bp-2Ch] BYREF
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
  sub_132FC(v8, (void **)va1);
  sub_132FC(v9, (void **)varg_r1a);
  sub_44004((LPCWSTR)a1);
  v2 = operator new(0x34u);
  if ( v2 != nullptr )
    v5 = sub_45EC0(v2, v3, v4);
  else
    v5 = 0;
  *(_DWORD *)(a1 + 1112) = v5;
  sub_46294(v5, 8, 7);
  *(_DWORD *)(*(_DWORD *)(a1 + 1112) + 24) = 0;
  *(_DWORD *)(*(_DWORD *)(a1 + 1112) + 4) = 1;
  *(_DWORD *)(*(_DWORD *)(a1 + 1112) + 8) = 118;
  *(_DWORD *)(*(_DWORD *)(a1 + 1112) + 28) = 1;
  v6 = *(_DWORD **)(a1 + 1112);
  if ( *v6 == 0 )
    __debugbreak();
  v6[9] = *(_DWORD *)(*v6 + 4);
  v7 = *(_DWORD **)(a1 + 1112);
  if ( *v7 == 0 )
    __debugbreak();
  v7[10] = *(_DWORD *)(*v7 + 8) / v7[1];
  sub_12F48((void **)varg_r1a);
  sub_12F48((void **)va1);
}

