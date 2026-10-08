/* ea:   0x00052A28 */
/* name: sub_52A28 */
/* size: 72 */
_DWORD *__fastcall sub_52A28(int a1, _DWORD *a2, int a3, _DWORD *a4)
{
  _DWORD *v7; // r3
  _DWORD *v8; // r0

  v7 = (_DWORD *)sub_52AA0(a1, 1);
  if ( v7 != (_DWORD *)-8 )
    v7[2] = *a4;
  v8 = *(_DWORD **)(a3 + 4);
  *v7 = a3;
  v7[1] = v8;
  *v8 = v7;
  *(_DWORD *)(a3 + 4) = v7;
  *a2 = v7;
  return a2;
}

