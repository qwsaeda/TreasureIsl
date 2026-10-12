/* ea:   0x0005B0CC */
/* name: sub_5B0CC */
/* size: 312 */
int __fastcall sub_5B0CC(_DWORD *a1, int a2, int a3)
{
  unsigned __int8 v6; // r4
  int v7; // r1
  bool v8; // zf
  int result; // r0
  int v10; // r0
  char *v11; // r6
  int v12; // r7
  size_t v13; // r5
  int v14; // r3
  char *i; // r4
  _DWORD v16[4]; // [sp+0h] [bp-4Ch] BYREF
  _DWORD v17[4]; // [sp+10h] [bp-3Ch] BYREF
  int v18; // [sp+20h] [bp-2Ch]
  int v19; // [sp+24h] [bp-28h]
  int v20; // [sp+28h] [bp-24h]
  int v21; // [sp+2Ch] [bp-20h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v6 = 0;
  v7 = a1[2];
  v16[1] = a1[1];
  v16[3] = v7;
  v16[0] = 0;
  v16[2] = 0;
  sub_5E9E0(v17, v16, a2);
  v8 = v18 == v19;
  if ( v18 == v19 )
    v8 = v21 == v20;
  if ( v8 )
    v6 = 1;
  result = v6;
  if ( v6 == 0 )
  {
    v10 = sub_5E970(a1, v20);
    v11 = (char *)(v10 + 2 * v18);
    v12 = v21 - v20;
    v13 = 2 * (v19 - v18);
    if ( a3 == 0 )
      __debugbreak();
    v14 = sub_5E970(a3, v17[2]);
    result = v17[0];
    for ( i = (char *)(v14 + 2 * v17[0]); v12 != 0; i += 2 * result )
    {
      memcpy(v11, i, v13);
      v11 += 2 * a1[1];
      if ( a3 == 0 )
        __debugbreak();
      result = *(_DWORD *)(a3 + 4);
      --v12;
    }
  }
  return result;
}

