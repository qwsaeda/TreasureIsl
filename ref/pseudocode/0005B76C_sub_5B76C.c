/* ea:   0x0005B76C */
/* name: sub_5B76C */
/* size: 404 */
int __fastcall sub_5B76C(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int v7; // r1
  unsigned __int8 v8; // r4
  bool v9; // zf
  int result; // r0
  int v11; // r0
  _WORD *v12; // r11
  int v13; // r5
  int v14; // r4
  int v15; // r10
  int v16; // r1
  int v17; // r7
  int v18; // r8
  int v19; // r5
  _WORD *i; // r4
  int v21; // t1
  int v23; // [sp+4h] [bp-60h]
  _DWORD v25[4]; // [sp+10h] [bp-54h] BYREF
  _DWORD v26[4]; // [sp+20h] [bp-44h] BYREF
  int v27; // [sp+30h] [bp-34h]
  int v28; // [sp+34h] [bp-30h]
  int v29; // [sp+38h] [bp-2Ch]
  int v30; // [sp+3Ch] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v7 = a1[2];
  v8 = 0;
  v25[1] = a1[1];
  v25[3] = v7;
  v25[0] = 0;
  v25[2] = 0;
  sub_5E9E0(v26, v25, a2);
  v9 = v27 == v28;
  if ( v27 == v28 )
    v9 = v30 == v29;
  if ( v9 )
    v8 = 1;
  result = v8;
  if ( v8 == 0 )
  {
    v11 = sub_5E970(a1, v29);
    v12 = (_WORD *)(v11 + 2 * v27);
    v13 = v30 - v29;
    v23 = v30 - v29;
    v14 = v28 - v27;
    if ( a3 == 0 )
      __debugbreak();
    v15 = sub_5E970(a3, v26[2]);
    if ( a3 == 0 )
      __debugbreak();
    v16 = *(_DWORD *)(a3 + 4);
    result = v26[0];
    if ( v13 != 0 )
    {
      v17 = 2 * (v16 - v26[0]);
      v18 = 2 * (v16 - v26[0] - v14);
      do
      {
        v19 = v17 + v15;
        for ( i = v12; v19 != v18 + v15; ++i )
        {
          v21 = *(unsigned __int16 *)(v19 - 2);
          v19 -= 2;
          *i = (*(int (__fastcall **)(int, int, _DWORD))(*(_DWORD *)a5 + 8))(a5, v21, (unsigned __int16)*i);
        }
        v12 += a1[1];
        result = a3;
        --v23;
        v15 += 2 * *(_DWORD *)(a3 + 4);
      }
      while ( v23 != 0 );
    }
  }
  return result;
}

