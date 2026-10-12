/* ea:   0x0005CA88 */
/* name: sub_5CA88 */
/* size: 420 */
int __fastcall sub_5CA88(_DWORD *a1, int a2, _DWORD *a3)
{
  unsigned __int8 v5; // r4
  int v6; // r1
  bool v7; // zf
  int result; // r0
  int v9; // r0
  _WORD *v10; // r9
  int v11; // r10
  int v12; // r8
  int v13; // r0
  unsigned __int8 *v14; // r7
  int v15; // r6
  _WORD *v16; // r5
  unsigned __int8 *i; // r3
  int v18; // t1
  _DWORD v20[4]; // [sp+8h] [bp-54h] BYREF
  _DWORD v21[4]; // [sp+18h] [bp-44h] BYREF
  int v22; // [sp+28h] [bp-34h]
  int v23; // [sp+2Ch] [bp-30h]
  int v24; // [sp+30h] [bp-2Ch]
  int v25; // [sp+34h] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == nullptr || a3[3] != 8 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v5 = 0;
  v6 = a1[2];
  v20[1] = a1[1];
  v20[3] = v6;
  v20[0] = 0;
  v20[2] = 0;
  sub_5E9E0(v21, v20, a2);
  v7 = v22 == v23;
  if ( v22 == v23 )
    v7 = v25 == v24;
  if ( v7 )
    v5 = 1;
  result = v5;
  if ( v5 == 0 )
  {
    v9 = sub_5E970(a1, v24);
    v10 = (_WORD *)(v9 + 2 * v22);
    v11 = v25 - v24;
    v12 = v23 - v22;
    v13 = sub_5E970(a3, v21[2]);
    v14 = (unsigned __int8 *)(v13 + v21[0]);
    if ( a3 == nullptr )
      __debugbreak();
    v15 = a3[8];
    result = sub_5E920(a3);
    if ( (_BYTE)result != 0 )
    {
      for ( ; v11 != 0; v14 += a3[1] )
      {
        v16 = v10;
        for ( i = v14; i != &v14[v12]; ++v16 )
        {
          v18 = *i++;
          *v16 = *(_WORD *)(v15 + 2 * v18);
        }
        result = a1[1];
        --v11;
        v10 += result;
      }
    }
    else
    {
      for ( ; v11 != 0; v14 += result )
      {
        sub_5EAF4(v14, &v14[v12], v10, v15);
        result = a3[1];
        --v11;
        v10 += a1[1];
      }
    }
  }
  return result;
}

