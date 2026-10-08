/* ea:   0x00053BC4 */
/* name: sub_53BC4 */
/* size: 388 */
int __fastcall sub_53BC4(_DWORD *a1, unsigned __int8 a2, int a3, unsigned __int16 a4)
{
  unsigned __int8 v5; // r5
  int v7; // r6
  int v8; // r4
  int v9; // r8
  int v10; // r4
  _DWORD *v11; // r4
  int v12; // t1
  int v13; // r3
  int v14; // r4
  int v15; // r4
  int v16; // t1
  int v17; // r3
  __int64 v18; // r0
  char v20; // [sp+0h] [bp-28h] BYREF

  v5 = a3;
  v7 = a2;
  v8 = a1[4] + 36 * a2 - 36;
  v9 = *(_DWORD *)sub_5467C(v8, &v20, a3);
  if ( v5 != 0 && *(_DWORD *)(*(_DWORD *)(v8 + 24) + 4 * v5 - 4) == 0x10000 )
    v9 /= 2;
  if ( v5 != 0 )
  {
    v10 = a1[1] + 8 * v5;
    v12 = *(_DWORD *)(v10 - 8);
    v11 = (_DWORD *)(v10 - 8);
    v13 = v12;
    if ( v12 != 0 )
    {
      if ( (unsigned int)(*(_DWORD *)(*a1 + 148) - v11[1]) >= 4 )
      {
        *v11 = 0;
      }
      else if ( v13 == 1 )
      {
        v9 *= 3;
      }
      else
      {
        v9 /= 3;
      }
    }
    v14 = a1[8] + 12 * v5;
    v16 = *(_DWORD *)(v14 - 12);
    v15 = v14 - 12;
    v17 = v16;
    if ( v16 != 0 && *(unsigned __int8 *)(v15 + 8) == v7 )
    {
      if ( (unsigned int)(*(_DWORD *)(*a1 + 148) - *(_DWORD *)(a1[1] + 8 * v5 - 4)) >= 4 )
      {
        *(_DWORD *)v15 = 0;
      }
      else if ( v17 == 1 )
      {
        v9 *= 3;
      }
      else
      {
        v9 /= 3;
      }
    }
  }
  LODWORD(v18) = 0;
  HIDWORD(v18) = (unsigned __int64)(int)(*(_DWORD *)(*a1 + 188) << 16) >> 16;
  return (int)((((int)(v18 / 6553600) * (__int64)v9) >> 16) * a4) >> 16;
}

