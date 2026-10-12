/* ea:   0x00036A9C */
/* name: sub_36A9C */
/* size: 436 */
int __fastcall sub_36A9C(int result, char *a2, unsigned int a3, int *a4)
{
  int v4; // r10
  char *v5; // r9
  char *v7; // r2
  int v8; // r6
  int v9; // r7
  unsigned int v10; // r4
  char *v11; // r8
  int v12; // r5
  char *v13; // r4
  char *v14; // r3
  int v15; // r1
  int v16; // r2
  int v17; // t1
  int v18; // r1
  _DWORD *v19; // r0
  unsigned int v20; // r1
  _DWORD *v21; // r3
  char *i; // r2
  int v23; // r1
  char v24; // [sp+8h] [bp-24h] BYREF

  v4 = result;
  v5 = a2;
  if ( a3 != 0 )
  {
    v7 = *(char **)(result + 4);
    if ( a3 > (*(_DWORD *)(result + 8) - (int)v7) >> 3 )
    {
      return sub_38260(result, a2, a4, &v24, a3, 0);
    }
    else
    {
      v8 = *a4;
      v9 = a4[1];
      v10 = (v7 - a2) >> 3;
      v11 = *(char **)(result + 4);
      if ( v10 <= a3 )
      {
        v19 = *(_DWORD **)(result + 4);
        if ( a3 != v10 )
        {
          v20 = a3 - v10;
          do
          {
            --v20;
            if ( v19 != nullptr )
            {
              *v19 = v8;
              v19[1] = v9;
            }
            v19 += 2;
          }
          while ( v20 != 0 );
        }
        v21 = (_DWORD *)(*(_DWORD *)(v4 + 4) + 8 * (a3 - v10));
        *(_DWORD *)(v4 + 4) = v21;
        for ( i = v5; i != v11; v21 += 2 )
        {
          if ( v21 != nullptr )
          {
            v23 = *((_DWORD *)i + 1);
            *v21 = *(_DWORD *)i;
            v21[1] = v23;
          }
          i += 8;
        }
        result = *(_DWORD *)(v4 + 4);
        for ( *(_DWORD *)(v4 + 4) = result + 8 * v10; v5 != v11; v5 += 8 )
        {
          *(_DWORD *)v5 = v8;
          *((_DWORD *)v5 + 1) = v9;
        }
      }
      else
      {
        v12 = 8 * a3;
        v13 = &v7[-v12];
        v14 = &v7[-v12];
        if ( &v7[-v12] != v7 )
        {
          do
          {
            if ( v7 != nullptr )
            {
              v15 = *((_DWORD *)v14 + 1);
              *(_DWORD *)v7 = *(_DWORD *)v14;
              *((_DWORD *)v7 + 1) = v15;
            }
            v14 += 8;
            v7 += 8;
          }
          while ( v14 != v11 );
        }
        *(_DWORD *)(result + 4) += v12;
        v16 = (v13 - v5) >> 3;
        if ( v16 > 0 )
        {
          do
          {
            v17 = *((_DWORD *)v13 - 2);
            v13 -= 8;
            --v16;
            v18 = *((_DWORD *)v13 + 1);
            *((_DWORD *)v11 - 2) = v17;
            v11 -= 8;
            *((_DWORD *)v11 + 1) = v18;
          }
          while ( v16 != 0 );
        }
        for ( result = (int)&v5[v12]; v5 != (char *)result; v5 += 8 )
        {
          *(_DWORD *)v5 = v8;
          *((_DWORD *)v5 + 1) = v9;
        }
      }
    }
  }
  return result;
}

