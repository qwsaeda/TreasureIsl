/* ea:   0x000380E8 */
/* name: sub_380E8 */
/* size: 48 */
_DWORD *__fastcall sub_380E8(_DWORD *a1, int a2, int a3)
{
  *a1 = 0;
  a1[1] = 0;
  a1[2] = 0;
  sub_13850(a1, a3 + 1);
  *(_WORD *)a1[1] = 0;
  return a1;
}

