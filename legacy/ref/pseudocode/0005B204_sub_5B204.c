/* ea:   0x0005B204 */
/* name: sub_5B204 */
/* size: 344 */
int __fastcall sub_5B204(_DWORD *a1, int a2, int a3)
{
  unsigned __int8 v6; // r4
  int v7; // r1
  bool v8; // zf
  int result; // r0
  int v10; // r0
  _WORD *v11; // r7
  int v12; // r4
  int v13; // r8
  int v14; // r6
  int v15; // r1
  int v16; // r3
  int v17; // r4
  int v18; // r5
  int v19; // r1
  _WORD *i; // r3
  __int16 v21; // t1
  _DWORD v22[4]; // [sp+0h] [bp-50h] BYREF
  _DWORD v23[4]; // [sp+10h] [bp-40h] BYREF
  int v24; // [sp+20h] [bp-30h]
  int v25; // [sp+24h] [bp-2Ch]
  int v26; // [sp+28h] [bp-28h]
  int v27; // [sp+2Ch] [bp-24h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v6 = 0;
  v7 = a1[2];
  v22[1] = a1[1];
  v22[3] = v7;
  v22[0] = 0;
  v22[2] = 0;
  sub_5E9E0(v23, v22, a2);
  v8 = v24 == v25;
  if ( v24 == v25 )
    v8 = v27 == v26;
  if ( v8 )
    v6 = 1;
  result = v6;
  if ( v6 == 0 )
  {
    v10 = sub_5E970(a1, v26);
    v11 = (_WORD *)(v10 + 2 * v24);
    v12 = v25 - v24;
    v13 = v27 - v26;
    if ( a3 == 0 )
      __debugbreak();
    v14 = sub_5E970(a3, v23[2]);
    if ( a3 == 0 )
      __debugbreak();
    v15 = *(_DWORD *)(a3 + 4);
    result = v23[0];
    v16 = v15 - v23[0] - v12;
    if ( v13 != 0 )
    {
      v17 = 2 * (v15 - v23[0]);
      v18 = 2 * v16;
      do
      {
        v19 = v17 + v14;
        for ( i = v11; v19 != v18 + v14; ++i )
        {
          v21 = *(_WORD *)(v19 - 2);
          v19 -= 2;
          *i = v21;
        }
        result = a1[1];
        --v13;
        v11 += result;
        v14 += 2 * *(_DWORD *)(a3 + 4);
      }
      while ( v13 != 0 );
    }
  }
  return result;
}

