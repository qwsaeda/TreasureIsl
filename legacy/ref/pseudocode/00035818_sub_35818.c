/* ea:   0x00035818 */
/* name: sub_35818 */
/* size: 516 */
int __fastcall sub_35818(int result, __int16 **a2, unsigned int a3, _DWORD *a4)
{
  int v4; // r10
  __int16 **v5; // r9
  int v7; // r8
  unsigned int v8; // r6
  int v9; // r7
  _DWORD *v10; // r6
  _DWORD *v11; // r5
  _DWORD *j; // r4
  int v13; // r5
  char *v14; // r4
  __int16 **k; // r4
  unsigned int v16; // r4
  _DWORD *v17; // r5
  __int16 **i; // r4
  char v19[8]; // [sp+8h] [bp-38h] BYREF
  __int16 *v20[12]; // [sp+10h] [bp-30h] BYREF

  v4 = result;
  v5 = a2;
  if ( a3 != 0 )
  {
    if ( (*(_DWORD *)(result + 8) - *(_DWORD *)(result + 4)) / 12 < a3 )
    {
      return sub_38118(result, a2, a4, v19, a3, 0);
    }
    else
    {
      sub_132FC(v20, a4);
      v7 = *(_DWORD *)(v4 + 4);
      v8 = (v7 - (int)v5) / 12;
      if ( v8 <= a3 )
      {
        v16 = a3 - v8;
        sub_38934(*(_DWORD *)(v4 + 4), v16, v20);
        v17 = (_DWORD *)(*(_DWORD *)(v4 + 4) + 12 * v16);
        *(_DWORD *)(v4 + 4) = v17;
        for ( i = v5; i != (__int16 **)v7; v17 += 3 )
        {
          if ( v17 != nullptr )
            sub_132FC(v17, i);
          i += 3;
        }
        for ( *(_DWORD *)(v4 + 4) += 12 * v8; v5 != (__int16 **)v7; v5 += 3 )
        {
          if ( v20 != v5 )
            sub_17890((int)v5, v20[0], v20[1]);
        }
      }
      else
      {
        v9 = 3 * a3;
        v10 = (_DWORD *)(v7 - 12 * a3);
        v11 = v10;
        for ( j = *(_DWORD **)(v4 + 4); v11 != (_DWORD *)v7; j += 3 )
        {
          if ( j != nullptr )
            sub_132FC(j, v11);
          v11 += 3;
        }
        *(_DWORD *)(v4 + 4) += v9 * 4;
        v13 = ((char *)v10 - (char *)v5) / 12;
        if ( v13 > 0 )
        {
          v14 = (char *)v10 - v7;
          do
          {
            v7 -= 12;
            sub_161B0(v7, (int)&v14[v7]);
            --v13;
          }
          while ( v13 != 0 );
        }
        for ( k = &v5[v9]; v5 != k; v5 += 3 )
        {
          if ( v20 != v5 )
            sub_17890((int)v5, v20[0], v20[1]);
        }
      }
      return sub_12EA0((int)v20);
    }
  }
  return result;
}

