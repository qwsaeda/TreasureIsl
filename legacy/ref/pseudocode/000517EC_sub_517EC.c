/* ea:   0x000517EC */
/* name: sub_517EC */
/* size: 340 */
int sub_517EC(int result, unsigned int a2, ...)
{
  int v2; // r10
  int v4; // r0
  _DWORD *v5; // r4
  unsigned int v6; // r7
  unsigned int v7; // r6
  unsigned int v8; // r5
  int v9; // r2
  unsigned int v10; // r0
  unsigned int v11; // r1
  int v12; // r0
  int v13; // r3
  int v14; // r2
  unsigned int v15; // r0
  int v16; // r0
  int v17; // r3
  int v18; // r1
  bool v19; // cf
  int v20; // r2
  int varg_r2; // [sp+2Ch] [bp-8h] BYREF
  va_list varg_r2a; // [sp+2Ch] [bp-8h]
  va_list varg_r3; // [sp+30h] [bp-4h] BYREF

  va_start(varg_r3, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(varg_r3, _DWORD);
  v2 = result;
  if ( varg_r2 <= 4 )
  {
    v4 = *(_DWORD *)(result + 24);
    v5 = *(_DWORD **)(v4 + 520);
    v6 = 0;
    result = sub_39948(v4 + 520, (int *)varg_r2a) == v5;
    if ( (_BYTE)result == 0 )
    {
      v7 = sub_16C9C((unsigned __int16 *)(v2 + 136), 2);
      v8 = *(_DWORD *)(*(_DWORD *)(v2 + 16) + 8);
      sub_16570((unsigned int *)(v2 + 144), a2);
      if ( a2 <= v8 + v7 )
      {
        if ( a2 > v7 )
        {
          v14 = *(_DWORD *)(v2 + 16);
          v15 = *(_DWORD *)(v14 + 8);
          if ( a2 - v7 <= v15 )
            v6 = v15 - (a2 - v7);
          *(_DWORD *)(v14 + 8) = v6;
        }
        v16 = sub_2F944((_DWORD *)(*(_DWORD *)(v2 + 24) + 520), (int *)varg_r2a);
        v17 = 999999;
        v18 = *(_DWORD *)(*(_DWORD *)v16 + 8);
        if ( v18 + a2 <= 0xF423F )
          v17 = v18 + a2;
        *(_DWORD *)(*(_DWORD *)v16 + 8) = v17;
      }
      else
      {
        v9 = *(_DWORD *)(v2 + 16);
        v10 = 0;
        v11 = *(_DWORD *)(v9 + 8);
        if ( v8 <= v11 )
          v10 = v11 - v8;
        *(_DWORD *)(v9 + 8) = v10;
        v12 = sub_2F944((_DWORD *)(*(_DWORD *)(v2 + 24) + 520), (int *)varg_r2a);
        v13 = 999999;
        if ( v8 + v7 + *(_DWORD *)(*(_DWORD *)v12 + 8) <= 0xF423F )
          v13 = v8 + v7 + *(_DWORD *)(*(_DWORD *)v12 + 8);
        *(_DWORD *)(*(_DWORD *)v12 + 8) = v13;
        *(_DWORD *)(v2 + 132) = a2 - v8 - v7;
        *(_BYTE *)(v2 + 4) = 0;
      }
      v19 = 3 * sub_16C9C((unsigned __int16 *)(v2 + 136), 2) >= 10 * a2;
      v20 = 1;
      if ( v19 )
        v20 = 2;
      return sub_34E28(*(_DWORD *)(v2 + 24), *(_DWORD *)(v2 + 8), v20);
    }
  }
  return result;
}

