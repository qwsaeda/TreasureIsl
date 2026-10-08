/* ea:   0x0005C4BC */
/* name: sub_5C4BC */
/* size: 496 */
int __fastcall sub_5C4BC(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int v8; // r0
  int v9; // r1
  bool v10; // zf
  int result; // r0
  int v12; // r0
  _WORD *v13; // r8
  int v14; // r5
  int v15; // r4
  int v16; // lr
  int v17; // r1
  int v18; // r5
  int v19; // r6
  _WORD *v20; // r9
  int v21; // r8
  int v22; // r10
  int v23; // r4
  int v24; // r1
  int v25; // t1
  _WORD *v26; // [sp+0h] [bp-6Ch]
  int v27; // [sp+4h] [bp-68h]
  int v28; // [sp+8h] [bp-64h]
  _DWORD v31[4]; // [sp+18h] [bp-54h] BYREF
  _DWORD v32[4]; // [sp+28h] [bp-44h] BYREF
  int v33; // [sp+38h] [bp-34h]
  int v34; // [sp+3Ch] [bp-30h]
  int v35; // [sp+40h] [bp-2Ch]
  int v36; // [sp+44h] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v8 = a1[1];
  v9 = a1[2];
  v31[0] = 0;
  v31[1] = v8;
  v31[2] = 0;
  v31[3] = v9;
  sub_5E9E0(v32, v31, a2);
  v10 = v33 == v34;
  if ( v33 == v34 )
    v10 = v36 == v35;
  LOBYTE(result) = v10;
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v12 = sub_5E970(a1, v35);
    v13 = (_WORD *)(v12 + 2 * v33);
    v14 = v36 - v35;
    v26 = v13;
    v28 = v36 - v35;
    v15 = v34 - v33;
    if ( a3 == 0 )
      __debugbreak();
    v16 = sub_5E970(a3, v32[2]);
    v27 = v16;
    if ( a3 == 0 )
      __debugbreak();
    v17 = *(_DWORD *)(a3 + 4);
    result = v32[0];
    if ( v14 != 0 )
    {
      v18 = 2 * (v17 - v32[0]);
      v19 = 2 * (v17 - v32[0] - v15);
      do
      {
        v20 = v13;
        v21 = v18 + v16;
        v22 = v19 + v16;
        if ( v19 + v16 != v18 + v16 )
        {
          v23 = (unsigned __int16)word_8122C;
          do
          {
            v25 = *(unsigned __int16 *)(v21 - 2);
            v21 -= 2;
            v24 = v25;
            if ( v23 != (unsigned __int16)v25 )
              *v20 = (*(int (__fastcall **)(int, int, _DWORD))(*(_DWORD *)a5 + 8))(a5, v24, (unsigned __int16)*v20);
            ++v20;
          }
          while ( v21 != v22 );
          v16 = v27;
        }
        result = a1[1];
        v13 = &v26[result];
        --v28;
        v16 += 2 * *(_DWORD *)(a3 + 4);
        v26 = v13;
        v27 = v16;
      }
      while ( v28 != 0 );
    }
  }
  return result;
}

