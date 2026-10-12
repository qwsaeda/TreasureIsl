/* ea:   0x0005CE34 */
/* name: sub_5CE34 */
/* size: 444 */
int __fastcall sub_5CE34(_DWORD *a1, int a2, _DWORD *a3)
{
  int v4; // r1
  unsigned __int8 v5; // r4
  bool v6; // zf
  int result; // r0
  int v8; // r0
  _WORD *v9; // r10
  int v10; // r11
  int v11; // r4
  int v12; // r9
  int v13; // r8
  int v14; // r6
  int v15; // r7
  int v16; // r3
  _WORD *i; // r5
  int v18; // t1
  _DWORD v21[4]; // [sp+8h] [bp-54h] BYREF
  _DWORD v22[4]; // [sp+18h] [bp-44h] BYREF
  int v23; // [sp+28h] [bp-34h]
  int v24; // [sp+2Ch] [bp-30h]
  int v25; // [sp+30h] [bp-2Ch]
  int v26; // [sp+34h] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == nullptr || a3[3] != 8 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v4 = a1[2];
  v5 = 0;
  v21[1] = a1[1];
  v21[3] = v4;
  v21[0] = 0;
  v21[2] = 0;
  sub_5E9E0(v22, v21, a2);
  v6 = v23 == v24;
  if ( v23 == v24 )
    v6 = v26 == v25;
  if ( v6 )
    v5 = 1;
  result = v5;
  if ( v5 == 0 )
  {
    v8 = sub_5E970(a1, v25);
    v9 = (_WORD *)(v8 + 2 * v23);
    v10 = v26 - v25;
    v11 = v24 - v23;
    v12 = sub_5E970(a3, v22[2]);
    if ( a3 == nullptr )
      __debugbreak();
    v13 = a3[8];
    v14 = a3[1] - v22[0];
    v15 = v14 - v11;
    result = sub_5E920(a3);
    if ( (_BYTE)result != 0 )
    {
      for ( ; v10 != 0; v12 += a3[1] )
      {
        v16 = v14 + v12;
        for ( i = v9; v16 != v15 + v12; ++i )
        {
          v18 = *(unsigned __int8 *)--v16;
          *i = *(_WORD *)(v13 + 2 * v18);
        }
        --v10;
        v9 += a1[1];
        result = (int)a3;
      }
    }
    else
    {
      for ( ; v10 != 0; v12 += result )
      {
        sub_5EBE0(v15 + v12, v14 + v12, v9, v13);
        result = a3[1];
        --v10;
        v9 += a1[1];
      }
    }
  }
  return result;
}

