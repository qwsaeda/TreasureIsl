/* ea:   0x0005BA80 */
/* name: sub_5BA80 */
/* size: 404 */
_DWORD *__fastcall sub_5BA80(_DWORD *a1, int a2, _DWORD *a3, int a4, int a5)
{
  int v7; // r1
  unsigned __int8 v8; // r4
  bool v9; // zf
  _DWORD *result; // r0
  int v11; // r0
  _WORD *v12; // r11
  int v13; // r5
  int v14; // r4
  int v15; // r0
  int v16; // r1
  int v17; // r10
  int v18; // r7
  int v19; // r8
  int v20; // r5
  _WORD *i; // r4
  int v22; // t1
  int v24; // [sp+4h] [bp-60h]
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
  v7 = a1[2];
  v8 = 0;
  v26[1] = a1[1];
  v26[3] = v7;
  v26[0] = 0;
  v26[2] = 0;
  sub_5E9E0(v27, v26, a2);
  v9 = v28 == v29;
  if ( v28 == v29 )
    v9 = v31 == v30;
  if ( v9 )
    v8 = 1;
  result = (_DWORD *)v8;
  if ( v8 == 0 )
  {
    v11 = sub_5E970(a1, v30);
    v12 = (_WORD *)(v11 + 2 * v28);
    v13 = v31 - v30;
    v24 = v31 - v30;
    v14 = v29 - v28;
    if ( a3 == nullptr )
      __debugbreak();
    v15 = sub_5E970(a3, a3[2] - v27[2] - 1);
    v16 = a3[1];
    v17 = v15;
    result = (_DWORD *)v27[0];
    if ( v13 != 0 )
    {
      v18 = 2 * (v16 - v27[0]);
      v19 = 2 * (v16 - v27[0] - v14);
      do
      {
        v20 = v18 + v17;
        for ( i = v12; v20 != v19 + v17; ++i )
        {
          v22 = *(unsigned __int16 *)(v20 - 2);
          v20 -= 2;
          *i = (*(int (__fastcall **)(int, int, _DWORD))(*(_DWORD *)a5 + 8))(a5, v22, (unsigned __int16)*i);
        }
        v12 += a1[1];
        result = a3;
        --v24;
        v17 -= 2 * a3[1];
      }
      while ( v24 != 0 );
    }
  }
  return result;
}

