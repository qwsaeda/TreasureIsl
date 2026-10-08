/* ea:   0x00060260 */
/* name: sub_60260 */
/* size: 48 */
_DWORD *__fastcall sub_60260(_DWORD *a1, _DWORD *a2, _DWORD *a3)
{
  *a1 = *a2;
  sub_132FC(a1 + 1, a2 + 1);
  a1[4] = *a3;
  return a1;
}

