/* ea:   0x000552A8 */
/* name: sub_552A8 */
/* size: 756 */
int __fastcall sub_552A8(int result, int a2, int a3)
{
  int v3; // r9
  int v5; // r2
  int v6; // r0
  int v7; // r8
  int v8; // r4
  int v9; // r3
  int v10; // r1
  int v11; // r6
  int v12; // r5
  int v13; // r4
  int v14; // r0
  char v15; // r2
  int v16; // r0
  int v17; // r1
  int v18; // r0
  int v19; // [sp+0h] [bp-24h] BYREF
  int v20; // [sp+4h] [bp-20h]

  v3 = result;
  if ( a2 != 1 )
  {
    result = *(_DWORD *)(result + 1060);
    v5 = *(_DWORD *)(v3 + 1060);
    if ( v5 == 0 || (result = *(_DWORD *)(v3 + 1064)) != 0 )
    {
      if ( a2 != 0 )
        return result;
      if ( a3 > 38 )
      {
        if ( a3 == 39 )
        {
          v18 = *(_DWORD *)(v3 + 1020);
          if ( v18 >= 6 && v18 <= 7 )
            goto LABEL_8;
          return sub_54D1C(v3, 1);
        }
        else
        {
          if ( a3 != 40 )
            return result;
          result = *(_DWORD *)(v3 + 1020);
          if ( result >= 6 && result <= 7 )
          {
            *(_DWORD *)(v3 + 1020) = 1;
            return result;
          }
          v17 = result + 1;
          result = *(_DWORD *)(v3 + 1064);
          *(_DWORD *)(v3 + 1020) = v17;
          if ( v17 > result )
            goto LABEL_43;
        }
      }
      else
      {
        switch ( a3 )
        {
          case 38:
            v16 = *(_DWORD *)(v3 + 1020);
            if ( v16 == 1 )
            {
LABEL_43:
              *(_DWORD *)(v3 + 1020) = 6;
              return 6;
            }
            if ( v16 <= 5 || v16 > 7 )
            {
              result = v16 - 1;
              *(_DWORD *)(v3 + 1020) = result;
            }
            else
            {
              result = *(_DWORD *)(v3 + 1064);
              *(_DWORD *)(v3 + 1020) = result;
            }
            break;
          case 13:
            result = *(_DWORD *)(v3 + 1020);
            if ( result == 6 )
            {
              *(_DWORD *)(v3 + 1140) = 1;
              *(_DWORD *)(v3 + 1008) = 1;
              v7 = 0;
              result = *(_DWORD *)(v3 + 1064);
              if ( v5 != 0 )
              {
                *(_DWORD *)(v3 + 1132) = 0;
                if ( result > 0 )
                {
                  v8 = v3;
                  do
                  {
                    v9 = *(_DWORD *)(v3 + 1028) + 8 * v7++;
                    *(_DWORD *)(v3 + 1024) += *(_DWORD *)(v8 + 1112) * *(_DWORD *)v9;
                    *(_WORD *)(v3 + 1052) += *(_WORD *)(v8 + 1112);
                    v10 = *(_DWORD *)(v8 + 1112);
                    v8 += 4;
                    *(_WORD *)(v9 + 4) -= v10;
                    result = *(_DWORD *)(v3 + 1064);
                  }
                  while ( v7 < result );
                }
              }
              else
              {
                v11 = 0;
                if ( result > 0 )
                {
                  v12 = v3;
                  do
                  {
                    v13 = *(_DWORD *)(v3 + 1040) + 8 * v11;
                    *(_DWORD *)(v3 + 1024) -= *(_DWORD *)(v12 + 1112) * *(_DWORD *)v13;
                    *(_WORD *)(v3 + 1052) -= *(_WORD *)(v12 + 1112);
                    v14 = *(_DWORD *)(v12 + 1112);
                    if ( v14 != 0 )
                    {
                      v20 = 0;
                      v19 = 0;
                      v15 = *(_BYTE *)(v13 + 6);
                      LOWORD(v20) = v14;
                      BYTE2(v20) = v15;
                      sub_4DCD8(v3 + 1028, &v19);
                    }
                    result = *(_DWORD *)(v3 + 1064);
                    ++v11;
                    v12 += 4;
                  }
                  while ( v11 < result );
                }
              }
            }
            else if ( result == 7 )
            {
              *(_DWORD *)(v3 + 1140) = 0;
              *(_DWORD *)(v3 + 1008) = 1;
              return 0;
            }
            break;
          case 37:
            v6 = *(_DWORD *)(v3 + 1020);
            if ( v6 < 6 || v6 > 7 )
              return sub_54D1C(v3, 0);
            goto LABEL_43;
          default:
            return result;
        }
      }
    }
    else
    {
      if ( a3 == 13 )
      {
        *(_DWORD *)(v3 + 1008) = 1;
        return 1;
      }
      if ( a3 == 37 || a3 == 39 )
      {
        if ( *(_DWORD *)(v3 + 1020) == 6 )
        {
LABEL_8:
          *(_DWORD *)(v3 + 1020) = 7;
          return 7;
        }
        goto LABEL_43;
      }
    }
  }
  return result;
}

