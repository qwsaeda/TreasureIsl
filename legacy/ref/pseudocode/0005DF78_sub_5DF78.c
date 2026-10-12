/* ea:   0x0005DF78 */
/* name: sub_5DF78 */
/* size: 480 */
int __fastcall sub_5DF78(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int v6; // r1
  unsigned __int8 v7; // r4
  bool v8; // zf
  int result; // r0
  int v10; // r0
  int v11; // r10
  int v12; // r11
  int v13; // r4
  int v14; // r9
  int v15; // r6
  int v16; // r7
  int v17; // r8
  __int16 v18; // r4
  __int16 v19; // [sp+8h] [bp-64h] BYREF
  __int16 v20; // [sp+Ah] [bp-62h] BYREF
  int v21; // [sp+Ch] [bp-60h]
  _DWORD *v22; // [sp+10h] [bp-5Ch]
  __int16 v23; // [sp+14h] [bp-58h]
  _DWORD v24[4]; // [sp+18h] [bp-54h] BYREF
  _DWORD v25[4]; // [sp+28h] [bp-44h] BYREF
  int v26; // [sp+38h] [bp-34h]
  int v27; // [sp+3Ch] [bp-30h]
  int v28; // [sp+40h] [bp-2Ch]
  int v29; // [sp+44h] [bp-28h]

  v21 = a3;
  v22 = a1;
  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 8 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v6 = a1[2];
  v7 = 0;
  v24[1] = a1[1];
  v24[3] = v6;
  v24[0] = 0;
  v24[2] = 0;
  sub_5E9E0(v25, v24, a2);
  v8 = v26 == v27;
  if ( v26 == v27 )
    v8 = v29 == v28;
  if ( v8 )
    v7 = 1;
  result = v7;
  if ( v7 == 0 )
  {
    v10 = sub_5E970(v22, v28);
    v11 = v10 + 2 * v26;
    v12 = v29 - v28;
    v13 = v27 - v26;
    v14 = sub_5E970(v21, v25[2]);
    if ( v21 == 0 )
      __debugbreak();
    v15 = *(_DWORD *)(v21 + 32);
    v16 = *(_DWORD *)(v21 + 4) - v25[0];
    v17 = v16 - v13;
    result = sub_5E920(v21);
    if ( (_BYTE)result != 0 )
    {
      if ( v12 != 0 )
      {
        v23 = (unsigned __int8)v19;
        v18 = (unsigned __int8)v19;
        do
        {
          v19 = v18;
          ((void (__fastcall *)(int, int, int, __int16 *, int, int))sub_5EF34)(v17 + v14, v16 + v14, v11, &v19, v15, a5);
          --v12;
          result = *(_DWORD *)(v21 + 4);
          v11 += 2 * v22[1];
          v14 += result;
        }
        while ( v12 != 0 );
      }
    }
    else if ( v12 != 0 )
    {
      v23 = (unsigned __int8)v19;
      do
      {
        v20 = (unsigned __int8)v19;
        sub_5EFBC(v17 + v14, v16 + v14, v11, &v20, v15, a5);
        --v12;
        result = *(_DWORD *)(v21 + 4);
        v11 += 2 * v22[1];
        v14 += result;
      }
      while ( v12 != 0 );
    }
  }
  return result;
}

