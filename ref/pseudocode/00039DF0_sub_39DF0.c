/* ea:   0x00039DF0 */
/* name: sub_39DF0 */
/* size: 40 */
_DWORD *__fastcall sub_39DF0(_DWORD *result, int *a2)
{
  _DWORD *v2; // r1
  int v3; // t1

  v3 = *a2;
  v2 = a2 + 1;
  *result = v3;
  result[1] = *v2;
  result[2] = v2[1];
  return result;
}

