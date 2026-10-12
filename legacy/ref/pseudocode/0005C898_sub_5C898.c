/* ea:   0x0005C898 */
/* name: sub_5C898 */
/* size: 496 */
int __fastcall sub_5C898(_DWORD *a1, int a2, _DWORD *a3, int a4, int a5)
{
  int v8; // r0
  int v9; // r1
  bool v10; // zf
  int result; // r0
  int v12; // r0
  _WORD *v13; // r8
  int v14; // r5
  int v15; // r4
  int v16; // r0
  int v17; // r1
  int v18; // lr
  int v19; // r5
  int v20; // r6
  _WORD *v21; // r9
  int v22; // r8
  int v23; // r10
  int v24; // r4
  int v25; // r1
  int v26; // t1
  _WORD *v27; // [sp+0h] [bp-6Ch]
  int v28; // [sp+4h] [bp-68h]
  int v29; // [sp+8h] [bp-64h]
  _DWORD v32[4]; // [sp+18h] [bp-54h] BYREF
  _DWORD v33[4]; // [sp+28h] [bp-44h] BYREF
  int v34; // [sp+38h] [bp-34h]
  int v35; // [sp+3Ch] [bp-30h]
  int v36; // [sp+40h] [bp-2Ch]
  int v37; // [sp+44h] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == nullptr || a3[3] != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v8 = a1[1];
  v9 = a1[2];
  v32[0] = 0;
  v32[1] = v8;
  v32[2] = 0;
  v32[3] = v9;
  sub_5E9E0(v33, v32, a2);
  v10 = v34 == v35;
  if ( v34 == v35 )
    v10 = v37 == v36;
  LOBYTE(result) = v10;
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v12 = sub_5E970(a1, v36);
    v13 = (_WORD *)(v12 + 2 * v34);
    v14 = v37 - v36;
    v27 = v13;
    v29 = v37 - v36;
    v15 = v35 - v34;
    if ( a3 == nullptr )
      __debugbreak();
    v16 = sub_5E970(a3, a3[2] - v33[2] - 1);
    v17 = a3[1];
    v18 = v16;
    result = v33[0];
    v28 = v18;
    if ( v14 != 0 )
    {
      v19 = 2 * (v17 - v33[0]);
      v20 = 2 * (v17 - v33[0] - v15);
      do
      {
        v21 = v13;
        v22 = v19 + v18;
        v23 = v20 + v18;
        if ( v20 + v18 != v19 + v18 )
        {
          v24 = (unsigned __int16)word_8122C;
          do
          {
            v26 = *(unsigned __int16 *)(v22 - 2);
            v22 -= 2;
            v25 = v26;
            if ( v24 != (unsigned __int16)v26 )
              *v21 = (*(int (__fastcall **)(int, int, _DWORD))(*(_DWORD *)a5 + 8))(a5, v25, (unsigned __int16)*v21);
            ++v21;
          }
          while ( v22 != v23 );
          v18 = v28;
        }
        result = a1[1];
        v13 = &v27[result];
        --v29;
        v18 -= 2 * a3[1];
        v27 = v13;
        v28 = v18;
      }
      while ( v29 != 0 );
    }
  }
  return result;
}

