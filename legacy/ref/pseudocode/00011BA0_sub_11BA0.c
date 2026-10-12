/* ea:   0x00011BA0 */
/* name: sub_11BA0 */
/* size: 660 */
int __fastcall sub_11BA0(_DWORD *a1)
{
  _DWORD *v1; // lr
  _DWORD *v2; // r1
  _DWORD *v3; // r8
  int result; // r0
  int v5; // r11
  _DWORD *v6; // r10
  _DWORD *v7; // r6
  int v8; // r0
  int (__fastcall *v9)(int, int, int, int, int); // r5
  _DWORD *v10; // r4
  int v11; // r1
  int v12; // r2
  int v13; // r1
  int v14; // r3
  int v15; // r4
  int v16; // [sp+Ch] [bp-54h]
  _DWORD *v18; // [sp+18h] [bp-48h]
  _DWORD v19[4]; // [sp+1Ch] [bp-44h] BYREF
  _DWORD v20[13]; // [sp+2Ch] [bp-34h] BYREF

  v1 = a1;
  v2 = (_DWORD *)a1[260];
  v3 = (_DWORD *)a1[259];
  result = 0;
  v18 = v2;
  v5 = 0;
  v16 = 0;
  if ( v3 != v2 )
  {
    v6 = v1;
    while ( 1 )
    {
      switch ( *v3 )
      {
        case 1:
          v7 = (_DWORD *)v1[71];
          goto LABEL_6;
        case 2:
          v7 = (_DWORD *)v1[70];
          goto LABEL_6;
        case 3:
          v7 = (_DWORD *)v1[74];
          goto LABEL_6;
        case 4:
          if ( v1[312] != 20 )
            goto LABEL_29;
          v10 = a1;
          if ( v1[315] - 1 != v5 )
            goto LABEL_31;
          result = v6[263] + 22;
          v16 = result;
          break;
        case 5:
          v7 = (_DWORD *)v1[72];
          goto LABEL_6;
        case 6:
          v7 = (_DWORD *)v1[73];
          goto LABEL_6;
        case 7:
          v7 = (_DWORD *)v1[68];
          goto LABEL_6;
        case 8:
          v7 = (_DWORD *)v1[67];
          goto LABEL_6;
        case 9:
          v7 = (_DWORD *)v1[69];
LABEL_6:
          if ( v7 != nullptr )
          {
            v8 = v7[3];
            if ( v8 == 16 )
            {
              v9 = sub_5C2CC;
            }
            else if ( v8 == 8 )
            {
              v9 = sub_5DBB0;
            }
            else
            {
              v9 = nullptr;
            }
            v1[122] = v9;
            v11 = v7[2];
            v19[1] = v7[1];
            v19[0] = 0;
            v19[2] = 0;
            v19[3] = v11;
            v12 = v6[263];
            v13 = v7[2];
            v14 = v6[264];
            v20[1] = v12 + v7[1];
            v15 = v1[1];
            v20[0] = v12;
            v20[2] = v14 - v13;
            v20[3] = v14;
            (*(void (__fastcall **)(int, _DWORD *, _DWORD *, _DWORD *, int (__fastcall *)(int, int, int, int, int), int *, int))(*(_DWORD *)v15 + 20))(
              v15,
              v20,
              v7,
              v19,
              v9,
              &dword_81230,
              1);
            v10 = a1;
            if ( a1[312] != 20 )
              goto LABEL_32;
            if ( a1[315] - 1 != v5 )
              goto LABEL_31;
            if ( *v3 == 4 )
            {
LABEL_32:
              result = v16;
            }
            else
            {
              result = v6[263] + (v7[1] >> 1);
              v16 = result;
            }
          }
          else
          {
            v10 = a1;
            result = v16;
          }
          break;
        default:
          if ( v1[312] == 20 )
          {
            v10 = a1;
            if ( v1[315] - 1 == v5 )
            {
              result = v6[263] + 22;
              v16 = result;
            }
            else
            {
LABEL_31:
              result = v16;
            }
          }
          else
          {
LABEL_29:
            v10 = a1;
            result = v16;
          }
          break;
      }
      ++v3;
      ++v5;
      v6 += 2;
      if ( v3 == v18 )
        break;
      v1 = a1;
    }
    if ( result != 0 )
      return sub_44648(v10, result);
  }
  return result;
}

