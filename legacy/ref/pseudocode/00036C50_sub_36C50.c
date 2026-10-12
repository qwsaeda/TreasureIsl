/* ea:   0x00036C50 */
/* name: sub_36C50 */
/* size: 528 */
int __fastcall sub_36C50(int result, char *a2, unsigned int a3, int *a4)
{
  int v4; // lr
  char *v5; // r11
  char *v8; // r3
  int v9; // r7
  int v10; // r8
  int v11; // r9
  char *v12; // r10
  unsigned int v13; // r5
  int v14; // r6
  char *v15; // r5
  char *v16; // r4
  int v17; // r1
  int v18; // r2
  int v19; // r3
  int v20; // t1
  int v21; // r1
  int v22; // r2
  unsigned int v23; // r2
  _DWORD *v24; // r0
  unsigned int v25; // r1
  char *v26; // r3
  _DWORD *v27; // r4
  int v28; // r1
  int v29; // r2
  char v30; // [sp+8h] [bp-28h] BYREF

  v4 = result;
  v5 = a2;
  if ( a3 != 0 )
  {
    v8 = *(char **)(result + 4);
    if ( (*(_DWORD *)(result + 8) - (int)v8) / 12 < a3 )
    {
      return sub_383BC(result, a2, a4, &v30, a3, 0);
    }
    else
    {
      v9 = *a4;
      v10 = a4[1];
      v11 = a4[2];
      v12 = *(char **)(result + 4);
      v13 = (v8 - a2) / 12;
      if ( v13 <= a3 )
      {
        v23 = a3 - v13;
        v24 = *(_DWORD **)(result + 4);
        if ( a3 != v13 )
        {
          v25 = a3 - v13;
          do
          {
            --v25;
            if ( v24 != nullptr )
            {
              *v24 = v9;
              v24[1] = v10;
              v24[2] = v11;
            }
            v24 += 3;
          }
          while ( v25 != 0 );
        }
        v26 = v5;
        v27 = (_DWORD *)(*(_DWORD *)(v4 + 4) + 12 * v23);
        for ( *(_DWORD *)(v4 + 4) = v27; v26 != v12; v27 += 3 )
        {
          if ( v27 != nullptr )
          {
            v28 = *((_DWORD *)v26 + 1);
            v29 = *((_DWORD *)v26 + 2);
            *v27 = *(_DWORD *)v26;
            v27[1] = v28;
            v27[2] = v29;
          }
          v26 += 12;
        }
        result = *(_DWORD *)(v4 + 4);
        for ( *(_DWORD *)(v4 + 4) = result + 12 * v13; v5 != v12; v5 += 12 )
        {
          *(_DWORD *)v5 = v9;
          *((_DWORD *)v5 + 1) = v10;
          *((_DWORD *)v5 + 2) = v11;
        }
      }
      else
      {
        v14 = 12 * a3;
        v15 = &v8[-12 * a3];
        v16 = v15;
        if ( v15 != v8 )
        {
          do
          {
            if ( v8 != nullptr )
            {
              v17 = *((_DWORD *)v16 + 1);
              v18 = *((_DWORD *)v16 + 2);
              *(_DWORD *)v8 = *(_DWORD *)v16;
              *((_DWORD *)v8 + 1) = v17;
              *((_DWORD *)v8 + 2) = v18;
            }
            v16 += 12;
            v8 += 12;
          }
          while ( v16 != v12 );
        }
        *(_DWORD *)(result + 4) += v14;
        v19 = (v15 - v5) / 12;
        if ( v19 > 0 )
        {
          do
          {
            v20 = *((_DWORD *)v15 - 3);
            v15 -= 12;
            --v19;
            v21 = *((_DWORD *)v15 + 1);
            v22 = *((_DWORD *)v15 + 2);
            *((_DWORD *)v12 - 3) = v20;
            v12 -= 12;
            *((_DWORD *)v12 + 1) = v21;
            *((_DWORD *)v12 + 2) = v22;
          }
          while ( v19 != 0 );
        }
        for ( result = (int)&v5[v14]; v5 != (char *)result; v5 += 12 )
        {
          *(_DWORD *)v5 = v9;
          *((_DWORD *)v5 + 1) = v10;
          *((_DWORD *)v5 + 2) = v11;
        }
      }
    }
  }
  return result;
}

