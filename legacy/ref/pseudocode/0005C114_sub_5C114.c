/* ea:   0x0005C114 */
/* name: sub_5C114 */
/* size: 440 */
int __fastcall sub_5C114(_DWORD *a1, int a2, _DWORD *a3)
{
  int v6; // r0
  int v7; // r1
  bool v8; // zf
  int result; // r0
  int v10; // r0
  unsigned __int16 *v11; // r10
  int v12; // r4
  int v13; // r11
  int v14; // r0
  int v15; // r1
  int v16; // r9
  int v17; // r3
  int v18; // r4
  int v19; // r3
  int v20; // r6
  unsigned __int16 *v21; // r7
  int v22; // r2
  unsigned __int16 v23; // r1
  unsigned __int16 v24; // t1
  char v25; // r0
  unsigned __int16 v26; // [sp+0h] [bp-64h]
  _DWORD v29[4]; // [sp+10h] [bp-54h] BYREF
  _DWORD v30[4]; // [sp+20h] [bp-44h] BYREF
  int v31; // [sp+30h] [bp-34h]
  int v32; // [sp+34h] [bp-30h]
  int v33; // [sp+38h] [bp-2Ch]
  int v34; // [sp+3Ch] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == nullptr || a3[3] != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v6 = a1[1];
  v7 = a1[2];
  v29[0] = 0;
  v29[1] = v6;
  v29[2] = 0;
  v29[3] = v7;
  sub_5E9E0(v30, v29, a2);
  v8 = v31 == v32;
  if ( v31 == v32 )
    v8 = v34 == v33;
  LOBYTE(result) = v8;
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v10 = sub_5E970(a1, v33);
    v11 = (unsigned __int16 *)(v10 + 2 * v31);
    v12 = v32 - v31;
    v13 = v34 - v33;
    if ( a3 == nullptr )
      __debugbreak();
    v14 = sub_5E970(a3, a3[2] - v30[2] - 1);
    v15 = a3[1];
    v16 = v14;
    result = v30[0];
    v17 = v15 - v30[0] - v12;
    if ( v13 != 0 )
    {
      v18 = 2 * (v15 - v30[0]);
      v19 = 2 * v17;
      do
      {
        v20 = v18 + v16;
        v21 = v11;
        if ( v19 + v16 != v18 + v16 )
        {
          v22 = (unsigned __int16)word_8122C;
          do
          {
            v24 = *(_WORD *)(v20 - 2);
            v20 -= 2;
            v23 = v24;
            v26 = v24;
            if ( v22 == v24 )
            {
              v25 = 0;
              v23 = v26;
            }
            else
            {
              v25 = 1;
            }
            if ( v25 != 0 )
              *v21 = v23;
            ++v21;
          }
          while ( v20 != v19 + v16 );
        }
        --v13;
        result = a1[1];
        v11 += result;
        v16 -= 2 * a3[1];
      }
      while ( v13 != 0 );
    }
  }
  return result;
}

