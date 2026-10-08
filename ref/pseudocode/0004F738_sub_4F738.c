/* ea:   0x0004F738 */
/* name: sub_4F738 */
/* size: 396 */
int sub_4F738(int a1, int a2, ...)
{
  int v4; // r5
  _DWORD *v5; // r4
  char v6; // r5
  int v7; // r3
  int v8; // r2
  _BYTE v10[4]; // [sp+0h] [bp-44h] BYREF
  int v11; // [sp+4h] [bp-40h] BYREF
  int v12; // [sp+8h] [bp-3Ch] BYREF
  int v13; // [sp+10h] [bp-34h] BYREF
  int v14; // [sp+14h] [bp-30h]
  int varg_r2; // [sp+3Ch] [bp-8h] BYREF
  va_list varg_r2a; // [sp+3Ch] [bp-8h]
  int varg_r3; // [sp+40h] [bp-4h]
  va_list va1; // [sp+44h] [bp+0h] BYREF

  va_start(va1, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _DWORD);
  v4 = varg_r3;
  sub_4EC88((_DWORD *)a1);
  *(_DWORD *)(a1 + 48) = a2;
  *(_DWORD *)(a1 + 56) = v4;
  *(_DWORD *)(a1 + 72) = 48;
  *(_DWORD *)(a1 + 52) = 0;
  *(_DWORD *)a1 = off_72088;
  *(_DWORD *)(a1 + 60) = 0;
  *(_DWORD *)(a1 + 64) = 0;
  *(_DWORD *)(a1 + 68) = 0;
  *(_BYTE *)(a1 + 76) = 0;
  v5 = (_DWORD *)*sub_1DDF0(a2 + 508, &v12);
  sub_50204(&v13, v10);
  if ( v4 == 3 )
  {
    v6 = 1;
    while ( v5 != *(_DWORD **)(*(_DWORD *)(a1 + 48) + 508) )
    {
      v7 = v5[5];
      if ( *(unsigned __int8 *)(v7 + 48) == (unsigned __int16)varg_r2 )
      {
        v11 = *(_DWORD *)(v7 + 4);
        sub_500B4(&v13, &v11);
      }
      v5 = sub_16A30(v5);
    }
    if ( v13 != v14 )
      v6 = 0;
    if ( v6 == 0 )
      *(_DWORD *)(a1 + 52) = *(_DWORD *)(v13 + 4 * ((unsigned int)(((v14 - v13) >> 2) + 1) >> 1));
  }
  else if ( v4 == 4 )
  {
    *(_DWORD *)(a1 + 52) = sub_5383C(*(_DWORD *)(a1 + 48) + 52, (unsigned __int16)varg_r2);
  }
  else
  {
    *(_DWORD *)(a1 + 52) = *(_DWORD *)(*(_DWORD *)(sub_39820(
                                                     (int *)(*(_DWORD *)(a1 + 48) + 508),
                                                     (unsigned __int16 *)varg_r2a)
                                                 + 20)
                                     + 4);
  }
  v8 = *(_DWORD *)(a1 + 52);
  if ( v8 != 0 )
  {
    *(_DWORD *)(a1 + 60) = 56 - *(_DWORD *)(v8 + 12);
    *(_DWORD *)(a1 + 64) = 94 - *(_DWORD *)(v8 + 16);
  }
  sub_50218(&v13);
  return a1;
}

