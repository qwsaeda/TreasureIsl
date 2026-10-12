/* ea:   0x0005DBB0 */
/* name: sub_5DBB0 */
/* size: 460 */
int __fastcall sub_5DBB0(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  unsigned __int8 v7; // r4
  int v8; // r1
  bool v9; // zf
  int result; // r0
  int v11; // r0
  int v12; // r9
  int v13; // r10
  int v14; // r8
  int v15; // r0
  int v16; // r7
  int v17; // r6
  __int16 v18; // r4
  __int16 v19; // [sp+8h] [bp-64h] BYREF
  __int16 v20; // [sp+Ah] [bp-62h] BYREF
  int v21; // [sp+Ch] [bp-60h]
  __int16 v22; // [sp+10h] [bp-5Ch]
  _DWORD v23[4]; // [sp+18h] [bp-54h] BYREF
  _DWORD v24[4]; // [sp+28h] [bp-44h] BYREF
  int v25; // [sp+38h] [bp-34h]
  int v26; // [sp+3Ch] [bp-30h]
  int v27; // [sp+40h] [bp-2Ch]
  int v28; // [sp+44h] [bp-28h]

  v21 = a3;
  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 8 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v7 = 0;
  v8 = a1[2];
  v23[1] = a1[1];
  v23[3] = v8;
  v23[0] = 0;
  v23[2] = 0;
  sub_5E9E0(v24, v23, a2);
  v9 = v25 == v26;
  if ( v25 == v26 )
    v9 = v28 == v27;
  if ( v9 )
    v7 = 1;
  result = v7;
  if ( v7 == 0 )
  {
    v11 = sub_5E970(a1, v27);
    v12 = v11 + 2 * v25;
    v13 = v28 - v27;
    v14 = v26 - v25;
    v15 = sub_5E970(v21, v24[2]);
    v16 = v15 + v24[0];
    if ( v21 == 0 )
      __debugbreak();
    v17 = *(_DWORD *)(v21 + 32);
    result = sub_5E920(v21);
    if ( (_BYTE)result != 0 )
    {
      if ( v13 != 0 )
      {
        v22 = (unsigned __int8)v19;
        v18 = (unsigned __int8)v19;
        do
        {
          v19 = v18;
          ((void (__fastcall *)(int, int, int, __int16 *, int, int))sub_5ED60)(v16, v16 + v14, v12, &v19, v17, a5);
          --v13;
          result = *(_DWORD *)(v21 + 4);
          v12 += 2 * a1[1];
          v16 += result;
        }
        while ( v13 != 0 );
      }
    }
    else if ( v13 != 0 )
    {
      v22 = (unsigned __int8)v19;
      do
      {
        v20 = (unsigned __int8)v19;
        sub_5EDEC(v16, v16 + v14, v12, &v20, v17, a5);
        --v13;
        result = *(_DWORD *)(v21 + 4);
        v12 += 2 * a1[1];
        v16 += result;
      }
      while ( v13 != 0 );
    }
  }
  return result;
}

