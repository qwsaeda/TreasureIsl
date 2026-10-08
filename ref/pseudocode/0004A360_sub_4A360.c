/* ea:   0x0004A360 */
/* name: sub_4A360 */
/* size: 72 */
_DWORD *__fastcall sub_4A360(int a1, _DWORD *a2, int a3, _DWORD *a4)
{
  _DWORD *v7; // r3
  _DWORD *v8; // r0

  v7 = (_DWORD *)sub_4A510(a1, 1);
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

