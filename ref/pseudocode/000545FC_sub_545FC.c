/* ea:   0x000545FC */
/* name: sub_545FC */
/* size: 60 */
_DWORD *__fastcall sub_545FC(_DWORD *a1, int a2)
{
  char *v4; // r0
  char *v5; // r1

  *a1 = 0;
  a1[1] = 0;
  a1[2] = 0;
  v4 = (char *)sub_4A7A8((int)(a1 + 2), a2);
  v5 = &v4[4 * a2];
  *a1 = v4;
  a1[1] = v4;
  a1[2] = v5;
  return a1;
}

