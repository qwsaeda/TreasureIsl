/* ea:   0x00054B7C */
/* name: sub_54B7C */
/* size: 284 */
void sub_54B7C(_DWORD *a1, ...)
{
  _DWORD *v2; // r4
  _DWORD *v3; // r0
  _DWORD *v4; // r0
  _DWORD *v5; // r0
  int v6; // r3
  _DWORD *v7; // r1
  _DWORD *v8; // r4
  _DWORD v9[4]; // [sp+Ch] [bp-3Ch] BYREF
  _DWORD v10[7]; // [sp+1Ch] [bp-2Ch] BYREF
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
  v2 = sub_132FC(v9, (void **)va1);
  v3 = sub_132FC(v10, (void **)varg_r1a);
  sub_44004(a1, *v3, v3[1], v3[2], *v2, v2[1], v2[2]);
  v4 = operator new(0x34u);
  if ( v4 != nullptr )
    v5 = sub_45EC0(v4);
  else
    v5 = nullptr;
  v6 = a1[27];
  a1[264] = v5;
  sub_46294(v5, 0, 0, v6);
  *(_DWORD *)(a1[264] + 24) = 0;
  *(_DWORD *)(a1[264] + 4) = 1;
  *(_DWORD *)(a1[264] + 8) = 25;
  *(_DWORD *)(a1[264] + 28) = 1;
  v7 = (_DWORD *)a1[264];
  if ( *v7 == 0 )
    __debugbreak();
  v7[9] = *(_DWORD *)(*v7 + 4);
  v8 = (_DWORD *)a1[264];
  if ( *v8 == 0 )
    __debugbreak();
  v8[10] = *(_DWORD *)(*v8 + 8) / v8[1];
  sub_12F48((void **)varg_r1a);
  sub_12F48((void **)va1);
}

