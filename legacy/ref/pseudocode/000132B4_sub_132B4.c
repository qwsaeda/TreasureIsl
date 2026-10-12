/* ea:   0x000132B4 */
/* name: sub_132B4 */
/* size: 20 */
void __fastcall sub_132B4(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

