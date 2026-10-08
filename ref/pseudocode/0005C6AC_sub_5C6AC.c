/* ea:   0x0005C6AC */
/* name: sub_5C6AC */
/* size: 492 */
int __fastcall sub_5C6AC(_DWORD *a1, int a2, _DWORD *a3, int a4, int a5)
{
  int v7; // r0
  int v8; // r1
  bool v9; // zf
  int result; // r0
  int v11; // r0
  int v12; // r8
  int v13; // r4
  int v14; // r5
  int v15; // r3
  unsigned __int16 *v16; // r11
  int v17; // r6
  int v18; // lr
  unsigned __int16 *v19; // r9
  int v20; // r8
  int v21; // r5
  int v22; // [sp+0h] [bp-64h]
  int v24; // [sp+8h] [bp-5Ch]
  _DWORD v26[4]; // [sp+10h] [bp-54h] BYREF
  _DWORD v27[4]; // [sp+20h] [bp-44h] BYREF
  int v28; // [sp+30h] [bp-34h]
  int v29; // [sp+34h] [bp-30h]
  int v30; // [sp+38h] [bp-2Ch]
  int v31; // [sp+3Ch] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == nullptr || a3[3] != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v7 = a1[1];
  v8 = a1[2];
  v26[0] = 0;
  v26[1] = v7;
  v26[2] = 0;
  v26[3] = v8;
  sub_5E9E0(v27, v26, a2);
  v9 = v28 == v29;
  if ( v28 == v29 )
    v9 = v31 == v30;
  LOBYTE(result) = v9;
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v11 = sub_5E970(a1, v30);
    v12 = v11 + 2 * v28;
    v13 = v29 - v28;
    v14 = v31 - v30;
    v22 = v12;
    v24 = v31 - v30;
    if ( a3 == nullptr )
      __debugbreak();
    v15 = sub_5E970(a3, a3[2] - v27[2] - 1);
    result = v27[0];
    v16 = (unsigned __int16 *)(v15 + 2 * v27[0]);
    if ( v14 != 0 )
    {
      v17 = v13;
      while ( 1 )
      {
        v18 = a5;
        v19 = v16;
        if ( v16 != &v16[v17] )
        {
          v20 = v12 - (_DWORD)v16;
          v21 = (unsigned __int16)word_8122C;
          while ( 1 )
          {
            if ( v21 != *v19 )
              *(unsigned __int16 *)((char *)v19 + v20) = (*(int (__fastcall **)(int, _DWORD, _DWORD))(*(_DWORD *)v18 + 8))(
                                                           v18,
                                                           *v19,
                                                           *(unsigned __int16 *)((char *)v19 + v20));
            if ( ++v19 == &v16[v17] )
              break;
            v18 = a5;
          }
          v14 = v24;
        }
        --v14;
        result = a1[1];
        v24 = v14;
        v22 += 2 * result;
        v16 -= a3[1];
        if ( v14 == 0 )
          break;
        v12 = v22;
      }
    }
  }
  return result;
}

