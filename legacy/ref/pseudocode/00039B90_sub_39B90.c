/* ea:   0x00039B90 */
/* name: sub_39B90 */
/* size: 52 */
_DWORD *__fastcall sub_39B90(int a1, _DWORD *a2)
{
  _DWORD *v3; // r3

  v3 = sub_34A70(a1, 1);
  if ( v3 != (_DWORD *)-16 )
  {
    v3[4] = *a2;
    v3[5] = a2[1];
  }
  return v3;
}

