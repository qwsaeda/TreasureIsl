/* ea:   0x00053A4C */
/* name: sub_53A4C */
/* size: 376 */
int __fastcall sub_53A4C(_DWORD *a1, int a2, unsigned __int8 a3, int a4, unsigned __int16 a5)
{
  unsigned __int8 v7; // r4
  int v8; // r6
  int v9; // r9
  int v10; // r5
  int v11; // r4
  _DWORD *v12; // r4
  int v13; // t1
  int v14; // r3
  int v15; // r4
  int v16; // r4
  int v17; // t1
  int v18; // r3
  _BYTE v20[40]; // [sp+0h] [bp-28h] BYREF

  v7 = a4;
  v8 = a3;
  v9 = *(_DWORD *)sub_5467C(a1[4] + 36 * a3 - 36, v20, a4);
  v10 = v7;
  if ( v7 != 0 )
  {
    v11 = a1[1] + 8 * v7;
    v13 = *(_DWORD *)(v11 - 8);
    v12 = (_DWORD *)(v11 - 8);
    v14 = v13;
    if ( v13 != 0 )
    {
      if ( (unsigned int)(*(_DWORD *)(*a1 + 148) - v12[1]) >= 4 )
      {
        *v12 = 0;
      }
      else if ( v14 == 1 )
      {
        v9 *= 3;
      }
      else
      {
        v9 /= 3;
      }
    }
  }
  if ( v10 != 0 )
  {
    v15 = a1[8] + 12 * v10;
    v17 = *(_DWORD *)(v15 - 12);
    v16 = v15 - 12;
    v18 = v17;
    if ( v17 != 0 && *(unsigned __int8 *)(v16 + 8) == v8 )
    {
      if ( (unsigned int)(*(_DWORD *)(*a1 + 148) - *(_DWORD *)(a1[1] + 8 * v10 - 4)) >= 4 )
      {
        *(_DWORD *)v16 = 0;
      }
      else if ( v18 == 1 )
      {
        v9 *= 3;
      }
      else
      {
        v9 /= 3;
      }
    }
  }
  if ( *(_DWORD *)(a2 + 176) == 7 )
    v9 /= 2;
  return (int)(((rt_sdiv64by64(0, (unsigned __int64)(int)(*(_DWORD *)(*a1 + 188) << 16) >> 16, 6553600, 0) * (__int64)v9) >> 16)
             * a5) >> 16;
}

