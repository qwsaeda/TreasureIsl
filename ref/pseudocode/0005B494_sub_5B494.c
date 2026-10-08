/* ea:   0x0005B494 */
/* name: sub_5B494 */
/* size: 344 */
int __fastcall sub_5B494(_DWORD *a1, int a2, _DWORD *a3)
{
  unsigned __int8 v6; // r4
  int v7; // r1
  bool v8; // zf
  int result; // r0
  int v10; // r0
  _WORD *v11; // r7
  int v12; // r4
  int v13; // r8
  int v14; // r0
  int v15; // r1
  int v16; // r6
  int v17; // r3
  int v18; // r4
  int v19; // r5
  int v20; // r1
  _WORD *i; // r3
  __int16 v22; // t1
  _DWORD v23[4]; // [sp+0h] [bp-50h] BYREF
  _DWORD v24[4]; // [sp+10h] [bp-40h] BYREF
  int v25; // [sp+20h] [bp-30h]
  int v26; // [sp+24h] [bp-2Ch]
  int v27; // [sp+28h] [bp-28h]
  int v28; // [sp+2Ch] [bp-24h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == nullptr || a3[3] != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v6 = 0;
  v7 = a1[2];
  v23[1] = a1[1];
  v23[3] = v7;
  v23[0] = 0;
  v23[2] = 0;
  sub_5E9E0(v24, v23, a2);
  v8 = v25 == v26;
  if ( v25 == v26 )
    v8 = v28 == v27;
  if ( v8 )
    v6 = 1;
  result = v6;
  if ( v6 == 0 )
  {
    v10 = sub_5E970(a1, v27);
    v11 = (_WORD *)(v10 + 2 * v25);
    v12 = v26 - v25;
    v13 = v28 - v27;
    if ( a3 == nullptr )
      __debugbreak();
    v14 = sub_5E970(a3, a3[2] - v24[2] - 1);
    v15 = a3[1];
    v16 = v14;
    result = v24[0];
    v17 = v15 - v24[0] - v12;
    if ( v13 != 0 )
    {
      v18 = 2 * (v15 - v24[0]);
      v19 = 2 * v17;
      do
      {
        v20 = v18 + v16;
        for ( i = v11; v20 != v19 + v16; ++i )
        {
          v22 = *(_WORD *)(v20 - 2);
          v20 -= 2;
          *i = v22;
        }
        result = a1[1];
        --v13;
        v11 += result;
        v16 -= 2 * a3[1];
      }
      while ( v13 != 0 );
    }
  }
  return result;
}

